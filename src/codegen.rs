use std::str;
use parser;
use parser::{Token, Type};
use code_writer::CodeWriter;

// convert from snake_case to CamelCase
pub fn rustify(underscores: &String, is_type: bool) -> String {
    let mut collect = String::from("");
    let chars: Vec<char> = underscores.chars().collect();
    let mut under = false;
    let mut first = true;
    let mut i = 0; // there is a more rusty way to do this
    let end_index = chars.len() - 1;
    for c in chars {
        if c == 't' && i == end_index && under {
            // don't push this guy, it's an annoying _t
        } else if c == '_' {
            under = true;
        } else if under || first {
            collect.push_str(c.to_uppercase().collect::<String>().as_str());
            first = false;
            under = false;
        } else if is_type {
            collect.push_str(c.to_lowercase().collect::<String>().as_str());
        } else {
            collect.push_str(c.to_string().as_str());
        }
        i += 1;
    }
    collect
}

fn convert_basic_token(ident: &Token, is_type: bool) -> String {
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
        Token::Ident(ref ty) => {
            if is_type {
                rustify(ty, is_type)
            } else {
                ty.clone()
            }
        },
        Token::Constant(ref val) => { val.to_string() },
        _ => { String::from("UNSUPORTED_TYPE") }
    };
    type_
}

fn write_struct(ident: Token, fields: Vec<Token>, wr: &mut CodeWriter) -> bool {
    let id = match ident {
        Token::Ident(ref id) => { rustify(id, false) },
        _ => { return false }
    };
    wr.pub_struct(id, |wr| {
        for field in fields.iter() {
            match *field {
                Token::Decl{ty: ref field_type, id: ref field_id} => {
                    wr.field_decl(
                        convert_basic_token(field_id, false).as_str(),
                        convert_basic_token(field_type, true).as_str());
                },
                Token::StringDecl{size: _, id: ref field_id} => {
                    wr.field_decl(
                        // TODO Manage sized strings
                        convert_basic_token(field_id, false).as_str(), "String");
                },
                _ => {
                    println!("UNIMPLEMENTED STRUCT FIELD");
                }
            };
        }
    });
    true
}

fn write_enum(ident: Token, fields: Vec<(Token, Token)>, wr: &mut CodeWriter) -> bool {
    let id = match ident {
        Token::Ident(ref id) => { rustify(id, false) },
        _ => { return false }
    };
    wr.pub_enum(id, |wr| {
        for &(ref field_id, ref field_val) in fields.iter() {
            match *field_val {
                Token::Blank => {
                    wr.enum_decl(convert_basic_token(field_id, false).as_str(), "");
                }
                _ => {
                    wr.enum_decl(
                        convert_basic_token(field_id, false).as_str(),
                        convert_basic_token(field_val, false).as_str());
                }
            }
        }
    });
    true
}

fn write_typedef(def: Token, wr: &mut CodeWriter) -> bool {
    match def {
        Token::VarArrayDecl{ty, id, size} => {
            wr.alias(convert_basic_token(&id, true), |wr| {
                wr.var_vec(convert_basic_token(&ty, true).as_str());
            });
        },
        Token::StringDecl{id, size} => {
            wr.alias(convert_basic_token(&id, true), |wr| {
                // TODO Size this somehow. maybe make these &[u8]
                wr.write(String::from("String"));
            });
        },
        Token::Decl{ty, id} => {
            wr.alias(convert_basic_token(&id, true), |wr| {
                wr.write(convert_basic_token(&ty, true).as_str());
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

fn write_version(procs: &Vec<Token>, wr: &mut CodeWriter) -> bool {
    wr.program_version_request(|wr| {
        for ptoken in procs {
            let (return_type, name, arg_types, id) = match *ptoken {
                Token::Proc{
                    ref return_type,
                    ref name,
                    ref arg_types,
                    ref id} => {
                    (return_type, name, arg_types, id)
                }, _ => { return; }
            };

            let mut arg_strings: Vec<String> = Vec::new();
            for arg in arg_types {
                match arg {
                    &Token::VoidDecl => {},
                    _ => {
                        arg_strings.push(convert_basic_token(arg, true));
                    }
                }
            }

            wr.version_proc_request(convert_basic_token(name.as_ref(), true).as_str(),
                &arg_strings);
        }
    });

    wr.program_version_response(|wr| {
        for ptoken in procs {
            let (return_type, name, arg_types, id) = match *ptoken {
                Token::Proc{
                    ref return_type,
                    ref name,
                    ref arg_types,
                    ref id} => {
                    (return_type, name, arg_types, id)
                }, _ => { return; }
            };

            let ret_str: Option<String> = match **return_type {
                Token::VoidDecl => None,
                _ => Some(convert_basic_token(return_type.as_ref(), true))
            };

            wr.version_proc_response(
                convert_basic_token(name.as_ref(), true).as_str(), ret_str);
        }
    });

    true
}

fn write_program(versions: &Vec<Token>, wr: &mut CodeWriter) -> bool {
    for vtoken in versions {
        let (name, id, procs) = match *vtoken {
            Token::Version{
                ref name,
                ref id,
                ref procs} => (name, id, procs),
            _ => { return false; }
        };

        if !write_version(&procs, wr) {
            return false;
        }
    }

    true
}

pub fn compile(wr: &mut CodeWriter, source: String) -> Result<&'static str, ()> {
    let bytes = source.into_bytes();
    let not_yet_parsed = bytes.as_slice();
    let tokens = parser::parse(not_yet_parsed, false);

    wr.write_header();

    for token in tokens.unwrap() {
        println!("{:?}", token);
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
            Token::Program{name, id, versions} => {
                write_program(&versions, wr);
            },
			_ => {
                println!("Codegen isn't supported for this token yet");
                //break
                // Err("Unsuported token")
            }
		}
	}

    Ok("Complete codegen")
}
