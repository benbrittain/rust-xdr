use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use std::str;
use std::io::{Write, Read};
use std::collections::HashMap;

use parser;
use parser::{Token, Type};
use code_writer::CodeWriter;

fn convert_basic_token(ident: &Token) -> String {
    let type_ = match *ident {
        Token::Type(ref ty) => {
            match *ty {
                Type::Uint   => { String::from("u32") },
                Type::Int    => { String::from("i32") },
                Type::Uhyper => { String::from("u64") },
                Type::Hyper  => { String::from("i64") },
                Type::Float  => { String::from("f32") },
                Type::Double => { String::from("f64") },
                Type::Bool   => { String::from("bool") },
                _ => { String::from("UNSUPORTED_TYPE") }
            }
        },
        Token::Ident(ref ty) => { ty.clone() },
        Token::Constant(ref val) => { val.to_string() },
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
                        convert_basic_token(field_id).as_str(),
                        convert_basic_token(field_type).as_str());
                },
                _ => { }
            };
        }
    });
    true
}

fn write_enum(ident: Token, fields: Vec<(Token, Token)>, wr: &mut CodeWriter) -> bool {
    let id = match ident {
        Token::Ident(ref id) => { id },
        _ => { return false }
    };
    wr.pub_enum(id, |wr| {
        for &(ref field_id, ref field_val) in fields.iter() {
            wr.enum_decl(
                convert_basic_token(field_id).as_str(),
                convert_basic_token(field_val).as_str());
        }
    });
    true
}

fn write_typedef(def: Token, wr: &mut CodeWriter) -> bool {
    match def {
        Token::VarArrayDecl{ty: ty, id: id, size: size} => {
            wr.alias(convert_basic_token(&id), |wr| {
                wr.var_vec(convert_basic_token(&ty).as_str());
            });
        },
        Token::StringDecl{id: id, size: size} => {
            wr.alias(convert_basic_token(&id), |wr| {
                // TODO Size this somehow. maybe make these &[u8]
                wr.write(String::from("String"));
            });
        },
        Token::Decl{ty, id} => {
            wr.alias(convert_basic_token(&id), |wr| {
                wr.write(convert_basic_token(&ty).as_str());
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

pub fn compile(wr: &mut CodeWriter, source: String) -> Result<&'static str, ()> {
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

            // These tokens are incredibly useful
            Token::StructDef{id, decl} => {
                match *decl {
                    Token::Struct(fields) => {
                        write_struct(*id, fields, wr);
                    },
                    _ => { println!("Unparsable") }
                };
            },
            Token::EnumDef{id, decl} => {
                match *decl {
                    Token::Enum(fields) => {
                        write_enum(*id, fields, wr);
                    },
                    _ => { println!("Unparsable") }
                };
            },
            Token::TypeDef(def) => {
                write_typedef(*def, wr);
            },
			_ => {
                println!("Codegen isn't supported for this token yet");
                break
                // Err("Unsuported token")
            }
		}
	}

    Ok("Complete codegen")
}
