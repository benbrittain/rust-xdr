use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use std::str;
use std::io::{Write, Read};
use std::collections::HashMap;

use parser;
use parser::{Token, Type};
use code_writer::CodeWriter;

fn convert_type(ident: &Box<Token>) -> String {
    let type_ = match **ident {
        Token::Type(ref ty) => {
            match *ty {
                Type::Uint   => { String::from("u32") },
                Type::Int    => { String::from("i32") },
                Type::Uhyper => { String::from("i32") },
                Type::Hyper  => { String::from("i64") },
                Type::Float  => { String::from("f32") },
                Type::Double => { String::from("f64") },
                Type::Bool   => { String::from("bool") },
                _ => { String::from("UNSUPORTED_TYPE") }
            }
        },
        Token::Ident(ref ty) => { ty.clone() },
        _ => { String::from("UNSUPORTED_TYPE") }
    };
    type_
}

fn write_struct(ident: Token, fields: Vec<Token>, wr: &mut CodeWriter) -> bool {
    let id = match ident {
        Token::Ident(ref id) => { id },
        _ => { return false }
    };
    wr.pub_struct(id, |wr| {
        for field in fields.iter() {
            match *field {
                Token::Decl{ty: ref field_type, id: ref field_id} => {
                    wr.field_decl(
                        convert_type(field_id).as_str(),
                        convert_type(field_type).as_str());
                },
                _ => { }
            };
        }
        // wr.comment("here");
    });
    true
}

fn write_typedef(def: Token, wr: &mut CodeWriter) -> bool {
    match def {
        Token::VarArrayDecl{ty: ty, id: id, size: size} => {
            wr.alias(convert_type(&id), |wr| {
                wr.var_vec(convert_type(&ty).as_str());
            });
        },
        _ => {
            println!("UNIMPLEMENTED TYPEDEF");
            println!("{:?}", def);
            return false
        }
    };
    true
}
    //    Token::StringDecl{id: id, size: size} => {
    //        match *id {
    //            Token::Ident(id) => {
    //                let ty_ = builder.ty().id("String");
    //                let item = builder.item().type_(id).build_ty(ty_);
    //                Some(item)
    //            },

pub fn compile(wr: &mut CodeWriter, path : &Path) -> Result<i32, ()> {
    // TODO clean this up
    let fin = File::open(path);
    let mut source = String::new();
    let _ = fin.unwrap().read_to_string(&mut source);
    let bytes = source.into_bytes();
    let mut not_yet_parsed = bytes.as_slice();
    let tokens = parser::parse(not_yet_parsed, false);

    wr.write_header();

    for token in tokens.unwrap() {
        match token {
            // These three tokens are useless to us, just ignore them
            Token::Blank => {},
            Token::Comment(_) => {},
            Token::CodeSnippet(_) => {}, // TODO maybe do something special with this
            Token::StructDef{id: id, decl: decl} => {
                match *decl {
                    Token::Struct(fields) => {
                        write_struct(*id, fields, wr);
                    },
                    _ => { println!("Unparsable") }
                };
            }
            Token::TypeDef(def) => {
                write_typedef(*def, wr);
                //codegen_typedef(wr, *def);
                //write!(wr, "test");
            },
			_ => { println!("Codegen isn't supported for this token yet"); }
		}
	}

    Ok(0)
}
