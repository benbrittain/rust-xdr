//use ast::{Block, Node};
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use std::str;

use nom::{IResult, ErrorKind, not_line_ending, space, alphanumeric, multispace};
use std::collections::HashMap;
use syntax;
use aster;

// Hey Ben, Where you left off
// you were going to move things into the Node enum, and try to make a unified codegen function
// you haven't tried parsing or gening Unions yet
// there probably is something more elegant I can do with the typing on the aster gen
// You were bouncing around ideas on how to integrate the functions, currently you think doing that
// at runtime might be a good idea, that's a bad idea think of something better
// mio might be good enough, consider using tokio-proto (like tim suggested), but you also kinda
// wanna keep the server/clnt dispatch from rpc_srv2
//
// So, finish the codegen for they types
// play around with Mio/tokio
// think of how to deal with the C priorly integrated into the proto files

enum Node<'a> {
    parsed_enum {
        id: &'a str,
        kv: Vec<(&'a str, &'a str)>
    }
}

#[derive(Debug)]
struct parsed_type<'a> {
    id: &'a str,
    _vec: Option<&'a [u8]>,
    _size: Option<&'a [u8]>,
}

impl<'a> parsed_type<'a> {
    fn is_vec(&self) -> bool {
        return match self._vec {
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

#[derive(Debug)]
struct parsed_typedef<'a> {
    id: parsed_type<'a>,
    new_id: parsed_type<'a>
}

//#[derive(Debug)]
//struct parsed_enum<'a> {
//    id: &'a str,
//    // TODO: make (str, uint)
//    kv: Vec<(&'a str, &'a str)>
//}

#[derive(Debug)]
struct parsed_struct<'a> {
    id: &'a str,
    args: Vec<(parsed_type<'a>, &'a str)>
}

named!(struct_id<&str>,
    map_res!(
        do_parse!(
            tag!("struct")              >>
            opt!(multispace)            >>
            name: take_until!(" ")      >>
            opt!(multispace)            >>
            tag!("{")                   >>
            opt!(multispace)            >>
            (name)
        ),
        str::from_utf8
    )
);

named!(struct_args<&[u8], (parsed_type, &str)>,
  do_parse!(
           opt!(multispace)                          >>
    type_: type_                                     >>
           opt!(multispace)                          >>
    id:    map_res!(
           take_until_and_consume!(";"),
           str::from_utf8
         )                                           >>
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
    key: map_res!(alphanumeric, str::from_utf8)	>>
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
		id: enum_id 		>>
		kv: many0!(enum_kv)	>>
			tag!("};") 		>>
		(Node::parsed_enum{id: id, kv: kv})
	)
);

named!(struct_<&[u8], parsed_struct>,
	do_parse!(
		id:     struct_id 		        >>
		args:   many0!(struct_args)	    >>
			    tag!("};") 		        >>
		(parsed_struct{id: id, args: args})
	)
);

named!(type_<&[u8], parsed_type>,
       do_parse!(
           id: map_res!(
                take_until_either!("< "),
                str::from_utf8)	>>
           is_vector: opt!(tag!("<"))          >>
           size: opt!(alphanumeric)          >>
           opt!(tag!(">"))          >>
           opt!(multispace)         >>
           (parsed_type{
               id: id,
               _size: size,
               _vec: is_vector
           })
       )
);

named!(typedef<&[u8], parsed_typedef>,
	do_parse!(
        id: tag!("typedef")     >>
        opt!(space)             >>
        left: type_             >>
        opt!(multispace)        >>
        right: type_            >>
        tag!(";") >>
        opt!(multispace) >>
        (parsed_typedef{id: left, new_id: right})
    )
);

fn codegen_enum(node: Node) -> syntax::ptr::P<syntax::ast::Item> {
    match node {
        Node::parsed_enum {id: id, kv: kv} => {
            let builder = aster::AstBuilder::new();
            let mut enum_ = builder.item().enum_(id);
            for (k, v) in kv {
                enum_ = enum_.id(k); // TODO build in the C style = v
            }
            enum_.build()
        }
    }
}

// Takes known types and converts them to their Rust equivelant
fn codegen_typedef(node: parsed_typedef) -> syntax::ptr::P<syntax::ast::Item> {
    let builder = aster::AstBuilder::new();
    // Cop out by not using codegen on the left val, TODO
    let expr_ = builder.item().type_(node.new_id.id).build_ty(codegen_type(node.id));
    return expr_;
}

// Takes known types and converts them to their Rust equivelant
fn codegen_type(node: parsed_type) -> syntax::ptr::P<syntax::ast::Ty> {
    let builder = aster::AstBuilder::new();
    let type_ = builder.ty();
    let core = match node.id {
        "int"       => type_.i32(),
        "float"     => type_.f32(),
        "string"    => type_.id("String"), // probably better way
        _           => type_.id(node.id),
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

fn codegen_struct(node: parsed_struct) -> syntax::ptr::P<syntax::ast::Item> {
    let builder = aster::AstBuilder::new();
    let mut struct_ = builder.item().struct_(node.id);
    let fields = node.args.into_iter().map(|(type_, ident)|
             aster::struct_field::StructFieldBuilder::named(ident).build_ty(codegen_type(type_)));
    return struct_.with_fields(fields).build();
}

pub fn compile(path : &Path) -> Result<i32, &str> {
    let fin = File::open(path);
    let mut source = String::new();
    let _ = fin.unwrap().read_to_string(&mut source);
    let bytes = source.into_bytes();
    let mut not_yet_parsed = bytes.as_slice();

    let builder = aster::AstBuilder::new();
    let mut block = builder.block();

    let res = typedef(not_yet_parsed);


    //match res {
    //    IResult::Done(unparsed, parsed) => {
    //        println!("{:?}", parsed);
    //        let block = block.stmt().build_item(codegen_typedef(parsed));
    //    }
    //    IResult::Error(err) => {
    //        //    println!("Error: {}", err);
    //    }
    //    IResult::Incomplete(_) => {}
    //}

    //let out = block.build();
    //println!("{}", syntax::print::pprust::block_to_string(&out));


    Ok(0)
}
