use std::str;
use parser;
use parser::{Token, Type};
use code_writer::CodeWriter;
use function_writer::*;
use std::collections::HashMap;

// convert from snake_case to CamelCase
pub fn rustify(underscores: &String) -> String {
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
        } else {
            collect.push_str(c.to_lowercase().collect::<String>().as_str());
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
                rustify(ty)
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
        Token::Ident(ref id) => { rustify(id) },
        _ => { return false }
    };
    wr.pub_struct(id, |wr| {
        for field in fields.iter() {
            match *field {
                Token::Decl{ty: ref field_type, id: ref field_id} => {
                    wr.pub_field_decl(
                        convert_basic_token(field_id, false).as_str(),
                        convert_basic_token(field_type, true).as_str());
                },
                Token::StringDecl{size: _, id: ref field_id} => {
                    wr.pub_field_decl(
                        // TODO Manage sized strings
                        convert_basic_token(field_id, false).as_str(), "String");
                },
                Token::VarArrayDecl{ref ty, ref id, ref size} => {
                    wr.pub_field_decl_fn(convert_basic_token(&id, false).as_str(), |wr| {
                        wr.var_vec(convert_basic_token(&ty, true).as_str());
                    });
                },
                      //  wr.pub_field_decl_fn(
                      //  wr.var_vec(convert_basic_token(&ty, true).as_str());
                      //  ty: Ident("example_uint_t"), id: Ident("my_vector_of_ints"), size: None 
                _ => {
                    println!("UNIMPLEMENTED STRUCT FIELD");
                }
            };
        }
    });
    true
}

fn write_union(ident: Token,
               ns_decl: &Box<Token>,
               cases: &Vec<Token>,
               default: &Box<Option<Token>>,
               tab: &mut SymbolTable,
               wr: &mut CodeWriter) -> bool {

    let id = match ident {
        Token::Ident(ref id) => { rustify(id) },
        _ => { return false }
    };
    let ns = match **ns_decl {
        Token::Decl{ref ty, ref id} => {
            ty
        }
        _=> { unreachable!()}
    };

    wr.pub_enum(id, |wr| {
        for arm in cases.iter() {
            match *arm {
                Token::UnionCase{ref vals, ref decl} => {
                    for case in vals.iter() {
                        match **decl {
                            Token::Decl{ref ty, ref id} => {
                                let token =  tab.add_symbol(ns, &case, id);
                                if let Some(t) = token {
                                    wr.enc_annotation(convert_basic_token(t, false).as_str());
                                }
                            },
                            _ => {}
                        };

                        wr.enum_struct_decl(convert_basic_token(case, true).as_str(), |wr| {
                            match **decl {
                                Token::Decl{ref ty, ref id} => {
                                    wr.field_decl(
                                        convert_basic_token(id, false).as_str(),
                                        convert_basic_token(ty, true).as_str());
                                },
                                _ => {}
                            };
                        });
                    }
                },
                _ => { }
            }
        }
        match **default{
            Some(ref token) => {
                wr.comment("Default case for the XDR Union");
                wr.enum_struct_decl("UnionDefault_", |wr| {
                    match *token {
                        Token::Decl{ref ty, ref id} => {
                            wr.field_decl(
                                convert_basic_token(id, false).as_str(),
                                convert_basic_token(ty, true).as_str());
                        },
                        Token::VoidDecl => {},
                        _ => { println!("Invalid AST!"); }
                    };
                });
            },
            None => {}
        }
    });
    true
}

fn write_enum(ident: Token,
              fields: Vec<(Token, Token)>,
              tab: &mut SymbolTable,
              wr: &mut CodeWriter) -> bool {

    let id = match ident {
        Token::Ident(ref id) => { rustify(id) },
        _ => { return false }
    };
    wr.pub_enum(id, |wr| {
        for &(ref field_id, ref field_val) in fields.iter() {
            match *field_val {
                Token::Blank => {
                    // Nothing to lookup here
                    wr.enum_decl(convert_basic_token(field_id, false).as_str(), "");
                }
                _ => {
                    //println!("enum:  {:?} {:?} {:?}", ident, field_id, field_val);
                    tab.add_symbol(&ident, field_id, field_val);
                    wr.enum_decl(
                        convert_basic_token(field_id, true).as_str(),
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
            wr.pub_alias(convert_basic_token(&id, true), |wr| {
                wr.var_vec(convert_basic_token(&ty, true).as_str());
            });
        },
        Token::StringDecl{id, size} => {
            wr.pub_alias(convert_basic_token(&id, true), |wr| {
                // TODO Size this somehow. maybe make these &[u8]
                wr.write(String::from("String"));
            });
        },
        Token::Decl{ty, id} => {
            wr.pub_alias(convert_basic_token(&id, true), |wr| {
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

fn write_version(prog_name: &String, ver_num: i64, procs: &Vec<Token>,
                 wr: &mut CodeWriter) -> bool {
    wr.pub_enum(&format!("{}RequestV{}", prog_name, ver_num), |wr| {
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

    wr.pub_enum(&format!("{}ResponseV{}", prog_name, ver_num), |wr| {
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

fn write_service_proc(prog_name: &String, ver_num: i64, proc_name: &Token,
                      arg_types: &Vec<Token>, wr: &mut CodeWriter) {
    let arg_names = (0..arg_types.len()).filter(|x| {
        match arg_types[*x] { Token::VoidDecl => { false}, _ => { true } }
    }).map(|x| { format!("arg{}", x) }).collect();
    wr.match_option(&format!("{}RequestV{}::{}", prog_name, ver_num,
        convert_basic_token(proc_name, true).as_str()), &arg_names, |wr| {
        wr.write(&format!("self.{}_v{}(",
            convert_basic_token(proc_name, false).as_str().to_lowercase(),
            ver_num));
        wr.comma_fields(&arg_names);
        wr.raw_write(")\n");
    });
}

fn write_service_version(prog_name: &String, ver_num: i64, procs: &Vec<Token>,
                         wr: &mut CodeWriter) {
    let version_fields = vec!["data"];
    wr.match_option(&format!("V{}", ver_num), &version_fields, |wr| {
        wr.match_block("data", |wr| {
            for ptoken in procs {
                if let Token::Proc{ref return_type, ref name, ref arg_types,
                        ref id} = *ptoken {
                    write_service_proc(prog_name, ver_num, (*name).as_ref(),
                        arg_types, wr);
                }
            }
        });
    });
}

fn write_service(prog_name: &String, versions: &Vec<Token>,
                 wr: &mut CodeWriter) -> bool {
    let rust_prog_name = rustify(prog_name);
    wr.program_version_service(&format!("{}Service", rust_prog_name), |wr| {
        wr.alias("Request", |wr| {
            wr.raw_write(&format!("{}Request", rust_prog_name));
        });
        wr.alias("Response", |wr| {
            wr.raw_write(&format!("{}Response", rust_prog_name));
        });
        wr.alias("Error", |wr| {
            wr.raw_write("io::Error");
        });
        wr.alias("Future", |wr| {
            wr.raw_write("BoxFuture<Self::Response, Self::Error>");
        });
        wr.dispatch_function(|wr| {
            wr.match_block("req", |wr| {
                for vtoken in versions {
                    if let Token::Version{ref name, ref id, ref procs} = *vtoken {
                        if let Token::Constant(id_num) = **id {
                            write_service_version(&rust_prog_name, id_num,
                                                  procs, wr);
                        }
                    }
                }
            });
        });
    });
    
    true
}

fn write_version_set(prog_name: &String, versions: &Vec<Token>,
                     set_type: &str, wr: &mut CodeWriter) -> bool {
    wr.pub_enum(&format!("{}{}", prog_name, set_type), |wr| {
        for vtoken in versions {
            if let Token::Version{ref name, ref id, ref procs} = *vtoken {
                if let Token::Constant(id_num) = **id {
                    wr.enum_tuple_decl(&format!("V{}", id_num), |w2| {
                        w2.raw_write(&format!("{}{}V{}", prog_name, set_type,
                                              id_num));
                    })
                }
            }
        }
    });

    true
}

fn write_program(prog_name: &String, versions: &Vec<Token>, wr: &mut CodeWriter) -> bool {
    let rust_prog_name = rustify(prog_name);

    write_version_set(&rust_prog_name, versions, "Request", wr);
    write_version_set(&rust_prog_name, versions, "Response", wr);

    for vtoken in versions {
        if let Token::Version{ref name, ref id, ref procs} = *vtoken {
            if let Token::Constant(id_num) = **id {
                if !write_version(&rust_prog_name, id_num, &procs, wr) {
                    return false;
                }
            }
        }
    }

    true
}

fn write_proc_decoder(prog_name: &String, ver_num: i64, proc_name: &Token,
                      ret_type: &Token, arg_types: &Vec<Token>,
                      wr: &mut CodeWriter) {
    let proc_decoder_fn = format!("{}_decode_v{}_{}",
        prog_name.to_lowercase(), ver_num,
        convert_basic_token(proc_name, false).as_str().to_lowercase());

    proc_decoder(&proc_decoder_fn, wr, |wr| {
        let mut i = 0u32;

        for atoken in arg_types {
            match *atoken {
                Token::VoidDecl => {},
                _ => {
                    proc_arg_decoder(i,
                        convert_basic_token(atoken, true).as_str(), wr);
                    i += 1;
                }
            }
        }

        let req_type = format!("{}RequestV{}", rustify(prog_name), ver_num);
        let req_name = convert_basic_token(proc_name, true);

        proc_decoder_finalize(&req_type, &req_name, i, wr);
    });
}

fn write_version_decoder(prog_name: &String, ver_num: i64, procs: &Vec<Token>,
                         wr: &mut CodeWriter) {
    let version_decoder_fn = format!("{}_decode_v{}",
        prog_name.to_lowercase(), ver_num);
    version_decoder(&version_decoder_fn, wr, |wr| {
        version_decoder_match(wr, |wr| {
            for ptoken in procs {
                if let Token::Proc{ref return_type,
                                   ref name,
                                   ref arg_types,
                                   ref id} = *ptoken {
                    let proc_decoder_fn = &format!("{}_{}",
                        &version_decoder_fn,
                        convert_basic_token(name.as_ref(), false).to_lowercase());
                    if let Token::Constant(id_num) = **id {
                        wr.match_option(&format!("{}u32", id_num),
                            &Vec::<String>::new(), |wr| {
                            proc_decoder_call(&proc_decoder_fn, wr);
                        });
                    }
               }
            }
            decoder_miss("procedure", wr);
        });

        version_decoder_finalize(rustify(prog_name), ver_num, wr);
    });

    for ptoken in procs {
        if let Token::Proc{ref return_type,
                           ref name,
                           ref arg_types,
                           ref id} = *ptoken {
            write_proc_decoder(prog_name, ver_num, name, return_type,
                               arg_types, wr);
       }
    }
}

fn write_decoder(prog_name: &String, prog_id: i64, versions: &Vec<Token>,
                 wr: &mut CodeWriter) -> bool {
    let prog_decoder_fn = format!("{}_decode", prog_name.to_lowercase());

    top_decoder(wr, |wr| {
        wr.match_block("header.program", |wr| {
            wr.match_option(&format!("{}u32", prog_id), &Vec::<String>::new(), |wr| {
                prog_decoder_call(&prog_decoder_fn, wr);
            });
            decoder_miss("program", wr);
        });
    });

    prog_decoder(&prog_decoder_fn, wr, |wr| {
        wr.match_block("version", |wr| {
            for vtoken in versions {
                if let Token::Version{ref name, ref id, ref procs} = *vtoken {
                    if let Token::Constant(id_num) = **id {
                        wr.match_option(&format!("{}u32", id_num),
                            &Vec::<String>::new(), |wr| {
                            version_decoder_call(&format!("{}_v{}",
                                &prog_decoder_fn, id_num), wr);
                        });
                    }
                }
            }
            decoder_miss("version", wr);
        });
    });

    for vtoken in versions {
        if let Token::Version{ref name, ref id, ref procs} = *vtoken {
            if let Token::Constant(id_num) = **id {
                write_version_decoder(prog_name, id_num, procs, wr);
            }
        }
    }

    true
}

fn write_version_encoder(prog_name: &String, ver_num: i64, procs: &Vec<Token>,
                         wr: &mut CodeWriter) {
    encoder_version(prog_name, ver_num, wr, |wr| {
        wr.match_block("rsp", |wr| {
            for ptoken in procs {
                if let Token::Proc{ref return_type,
                                   ref name,
                                   ref arg_types,
                                   ref id} = *ptoken {
                    encoder_proc(prog_name,
                                 convert_basic_token(name, true).as_str(),
                                 ver_num, wr);
                }
            }
            decoder_miss("procedure", wr);
        });
    });
}

fn write_encoder(prog_name: &String, versions: &Vec<Token>,
                 wr: &mut CodeWriter) -> bool {
    let rust_prog_name = rustify(prog_name);

    encoder(&rust_prog_name, wr, |wr| {
        wr.match_block("msg", |wr| {
            for vtoken in versions {
                if let Token::Version{ref name, ref id, ref procs} = *vtoken {
                    if let Token::Constant(id_num) = **id {
                        write_version_encoder(&rust_prog_name, id_num, procs,
                                              wr);
                    }
                }
            }
            decoder_miss("version", wr);
        });
    });
    true
}

fn write_namespace(name: &String, progs: &Vec<Token>, wr: &mut CodeWriter) -> bool {
    wr.namespace(name, |wr| {
        for ptoken in progs {
            if let Token::Program{ref name, ref id, ref versions} = *ptoken {
                if let Token::Ident(ref name_str) = **name{
                    write_program(name_str, &versions, wr);
                    write_service(name_str, &versions, wr);
                    if let Token::Constant(id_num) = **id {
                        write_decoder(name_str, id_num, versions, wr);
                        write_encoder(name_str, versions, wr);
                    }
                    ()
                }
            }
        }
    });

    true
}

#[derive(Debug)]
struct SymbolTable<'a> {
    //table: HashMap<&'a str, User<'a>>,
    name: &'a str,
    table: HashMap<(Token, Token), Token>,
}

impl<'a> SymbolTable<'a> {
    fn new(name: &str) -> SymbolTable {
        let table = HashMap::new();
        SymbolTable {
            name: name,
            table: table,
        }
    }
    fn add_symbol(&mut self, ns_tkn: &Token, id_tkn: &Token, val: &Token) -> Option<&Token> {
        let key = (ns_tkn.clone(), id_tkn.clone());
        if self.table.contains_key(&key) {
            // Don't overwrite symbol, right?
            //println!("Table already contains the token");
            self.table.get(&key)
        } else {
            self.table.insert(key, val.clone());
            None
        }
    }
}

pub fn compile(wr: &mut CodeWriter, source: String) -> Result<&'static str, ()> {
    let bytes = source.into_bytes();
    let not_yet_parsed = bytes.as_slice();
    let tokens = parser::parse(not_yet_parsed, false);

    let mut tab = SymbolTable::new("CodegenTable");

    wr.write_header();

    for token in tokens.unwrap() {
    //    println!("{:?}", token);
        match token {
            // These three tokens are useless to us, just ignore them
            Token::Blank => {},
            Token::Comment(_) => {},
            Token::CodeSnippet(_) => {}, // TODO maybe do something special with this

            // These tokens are incredibly useful
            Token::UnionDef{id, decl} => {
                match *decl {
                    Token::Union{decl: ref decl, ref cases, ref default} => {
                        write_union(*id, decl, cases, default, &mut tab, wr);
                    },
                    _ => { println!("Unparsable") }
                };
            },
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
                        write_enum(*id, fields, &mut tab, wr);
                    },
                    _ => { println!("Unparsable") }
                };
            },
            Token::TypeDef(def) => {
                write_typedef(*def, wr);
            },
            Token::Program{name, id, versions} => {
                match *name {
                    Token::Ident(ref name_str) => {
                        write_program(name_str, &versions, wr);
                    },
                    _ => { println!("Unparsable") }
                }
            },
            Token::Namespace{name, progs} => {
                match *name {
                    Token::Ident(ref s) => {
                        write_namespace(s, &progs, wr);
                    },
                    _ => { println!("Unparsable") }
                }
            }
			_ => {
                println!("Codegen isn't supported for this token yet");
                //break
                // Err("Unsuported token")
            }
		}
	}

    Ok("Complete codegen")
}
