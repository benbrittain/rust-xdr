//use ast::{Block, Node};
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use std::str;

use nom::{IResult, ErrorKind, not_line_ending, space, alphanumeric, multispace};
use std::collections::HashMap;
use syntax;
use aster;

use parser;

// Hey Ben, Where you left off
// you were going to move things into the Node enum, and try to make a unified codegen function
// you haven't tried parsing or gening Unions yet
// Unions are hard, what do they represent as rust types exactly?! fancy enums?
// there probably is something more elegant I can do with the typing on the aster gen
// You were bouncing around ideas on how to integrate the functions, currently you think doing that
// at runtime might be a good idea, that's a bad idea think of something better
// mio might be good enough, consider using tokio-proto (like tim suggested), but you also kinda
// wanna keep the server/clnt dispatch from rpc_srv2, proto manages multiplexing for you (I want optional
// passthrough)
//
// So, finish the codegen for they types
// play around with Mio/tokio
// think of how to deal with the C priorly integrated into the proto files

#[derive(Debug)]
enum Node<'a> {
    parsed_enum {
        id: &'a str,
        kv: Vec<(&'a str, &'a str)>
    },
	parsed_struct {
		id: &'a str,
		args: Vec<(parsed_type<'a>, &'a str)>
	},
	parsed_typedef {
		id: parsed_type<'a>,
		new_id: parsed_type<'a>
	},
}

#[derive(Debug)]
struct parsed_type<'a> {
    id: &'a str,
    _vec: Option<&'a [u8]>,
    _size: Option<&'a [u8]>,
    _sign: Option<&'a [u8]>,
    _hyper: Option<&'a [u8]>,
}

impl<'a> parsed_type<'a> {
    fn is_vec(&self) -> bool {
        return match self._vec {
            Some(_) => true,
            None => false
        };
    }
    fn is_unsigned(&self) -> bool {
        return match self._sign {
            Some(_) => true,
            None => false
        };
    }
    fn is_hyper(&self) -> bool {
        return match self._hyper {
            Some(_) => true,
            None => false
        };
    }
    fn size(&self) -> Option<usize> {
        return match self._size {
            Some(size) => Some(size[0] as usize),
            None => None
        };
    }
}


named!(struct_id<&str>,
    map_res!(
        do_parse!(
            tag!("struct")              >>
            opt!(multispace)            >>
            name: take_until!(" ")      >>
            (name)
        ),
        str::from_utf8
    )
);

named!(union_id<&str>,
    map_res!(
        do_parse!(
                    tag!("union")               >>
                    opt!(multispace)            >>
            name:   take_until!(" ")            >>
                    opt!(multispace)            >>
                    tag!("switch")              >>
                    opt!(multispace)            >>
                    tag!("(")                   >>
            s_type: type_                       >>
            s_name: take_until!(")")            >>
                    tag!(")")                   >>
                    opt!(multispace)            >>
                    tag!("{")                   >>
            (name)
        ),
        str::from_utf8
    )
);

named!(struct_args<&[u8], (parsed_type, &str)>,
  do_parse!(
    type_: type_                                     >>
           opt!(multispace)                          >>
    id:    map_res!(
           		take_until_either!(" ;"),
           		str::from_utf8
           )                                         >>
         opt!(multispace)                            >>
         tag!(";")									 >>
         opt!(multispace)                            >>
    (type_, id)
  )
);

named!(enum_id<&str>,
    map_res!(
        do_parse!(
            tag!("enum")            >>
            opt!(multispace)        >>
            name: take_until!(" ")  >>
            opt!(multispace)        >>
            tag!("{")               >>
            opt!(multispace)        >>
            (name)
        ),
        str::from_utf8
    )
);

named!(enum_kv<&[u8],(&str,&str)>,
  do_parse!(
         opt!(multispace)                            >>
    key: map_res!(take_until!(" "), str::from_utf8)	>>
         opt!(space)                                 >>
         tag!("=")                                   >>
         opt!(space)                                 >>
    val: map_res!(
           take_until_either_and_consume!("\n,"),
           str::from_utf8
         )                                           >>
         opt!(multispace)                            >>
    (key, val)
  )
);

named!(enumerator<&[u8], Node>,
	do_parse!(
		opt!(multispace)	>>
		id: enum_id 		>>
		kv: many0!(enum_kv)	>>
			tag!("};") 		>>
            multispace      >>
		(Node::parsed_enum{id: id, kv: kv})
	)
);

named!(struct_<&[u8], Node>,
    do_parse!(
				opt!(multispace)		>>
        id:     struct_id               >>
				opt!(multispace)		>>
				tag!("{")				>>
				opt!(multispace)		>>
        args:   many0!(struct_args)     >>
                tag!("};")              >>
				opt!(multispace)		>>
        (Node::parsed_struct{id: id, args: args})
    )
);

//named!(union<&[u8], &str>,
//    do_parse!(
//				opt!(multispace)		>>
//        id:     union_id                >>
//				opt!(multispace)		>>
//        (id.id)
//    )
//);


named!(type_<&[u8], parsed_type>,
       do_parse!(
           sign: opt!(tag!("unsigned"))     >>
           opt!(multispace)                 >>
           hyper: opt!(tag!("hyper"))       >>
           opt!(multispace)                 >>
           id: map_res!(
                take_until_either!("< "),
                str::from_utf8)	>>
           is_vector: opt!(tag!("<"))       >>
           size: opt!(alphanumeric)         >>
           opt!(tag!(">"))                  >>
           opt!(multispace)                 >>
           (parsed_type{
               id: id,
               _hyper: hyper,
               _sign: sign,
               _size: size,
               _vec: is_vector
           })
       )
);

named!(typedef<&[u8], Node>,
	do_parse!(
        id: tag!("typedef")     >>
        opt!(space)             >>
        left: type_             >>
        opt!(multispace)        >>
        right: type_            >>
        tag!(";") >>
        opt!(multispace) >>
        (Node::parsed_typedef{id: left, new_id: right})
    )
);

fn codegen_statement(node: Node) -> syntax::ptr::P<syntax::ast::Item> {
    match node {
        Node::parsed_enum {id: id, kv: kv} => {
            let builder = aster::AstBuilder::new();
            let mut enum_ = builder.item().enum_(id);
            for (k, v) in kv {
                enum_ = enum_.id(k); // TODO build in the C style = v
            }
            enum_.build()
        },
		Node::parsed_struct {id: id, args: args} => {
			let builder = aster::AstBuilder::new();
			let mut struct_ = builder.item().struct_(id);
			let fields = args.into_iter().map(|(type_, ident)|
				aster::struct_field::StructFieldBuilder::named(ident).build_ty(codegen_type(type_)));
			return struct_.with_fields(fields).build();
		},

		Node::parsed_typedef {id: id, new_id: new_id} => {
			let builder = aster::AstBuilder::new();
			// Cop out by not using codegen on the left val, TODO
			let expr_ = builder.item().type_(new_id.id).build_ty(codegen_type(id));
			return expr_;
		}
	}
}

// Takes known types and converts them to their Rust equivelant
fn codegen_type(node: parsed_type) -> syntax::ptr::P<syntax::ast::Ty> {
    let builder = aster::AstBuilder::new();
    let type_ = builder.ty();
    let core = match node.id {
        "int"                => {
            if node.is_unsigned() && node.is_hyper() {
                type_.u64()
            } else if !node.is_unsigned() && node.is_hyper() {
                type_.i64()
            } else if node.is_unsigned() && !node.is_hyper() {
                type_.u32()
            } else {
                type_.i32()
            }
        }
        "float"              => type_.f32(),
        "double"             => type_.f64(),
//      "quadruple"          => type_.f128(), // TODO
        "string"             => type_.id("String"), // probably better way
        _                    => type_.id(node.id),
    };

    if node.is_vec() {
        return match node.size() {
            Some(size) => builder.ty().build_array(core, size),
            None => builder.ty().build_slice(core)
        };
    } else {
        core
    }
}

named!(statements<&[u8], Vec<Node> >, many0!(alt!(enumerator | struct_ | typedef)));

pub fn compile(path : &Path) -> Result<i32, &str> {
    let fin = File::open(path);
    let mut source = String::new();
    let _ = fin.unwrap().read_to_string(&mut source);

    let bytes = source.into_bytes();
    let mut not_yet_parsed = bytes.as_slice();
    let tokens = parser::parse(not_yet_parsed, true);



    //let builder = aster::AstBuilder::new();
    //let mut block = builder.block();


    //let res = statements(not_yet_parsed);
    ////let res = union_id(not_yet_parsed);

    //match res {
    //    IResult::Done(unparsed, parsed) => {
    //        println!("{:?}", parsed);
	//		let out = str::from_utf8(&unparsed).unwrap();
    //        println!("{:?}", out);
	//		//let block = block.stmt().item().type_("test").build_ty(codegen_type(parsed));
    //        //println!("{}", syntax::print::pprust::block_to_string(&block.build()));
    //    }
    //    IResult::Error(err) => {
    //            println!("Error: {}", err);
    //    }
    //    IResult::Incomplete(needed) => {
	//		println!("Incomplete {:?}", needed);
	//	}
    //}

    //let out = block.build();
    //println!("{}", syntax::print::pprust::block_to_string(&out));


    Ok(0)
}
