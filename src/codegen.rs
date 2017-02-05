use std::str;
use std::collections::HashMap;
use code_writer::CodeWriter;
use function_writer::*;
use parser::{self, Token, Type};

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
        //_ => { String::from("UNSUPORTED_TYPE") }
        _ => { format!("UNSUPORTED TYPE: {:?}", ident) }
    };
    type_
}

fn write_struct(ident: &Box<Token>,
                fields: &Vec<Token>,
                mut tab: &mut CodegenState,
                wr: &mut CodeWriter) -> bool {
    let id = match **ident {
        Token::Ident(ref id) => { rustify(id) },
        _ => { return false }
    };
    wr.pub_struct(id, |wr| {
        for field in fields.iter() {
            match *field {
                Token::Decl{ty: ref field_type, id: ref field_id} => {
                    match **field_type {
                        Token::Union{decl: ref decl, ref cases, ref default} => {
                            wr.pub_field_decl(
                                convert_basic_token(field_id, false).as_str(),
                                convert_basic_token(field_id, true).as_str());
                            tab.hoist(&Token::UnionDef{
                                id: Box::new(*field_id.clone()),
                                decl: Box::new(*field_type.clone()),
                            });
                        },
                        Token::Struct(_) => {
                            wr.pub_field_decl(
                                convert_basic_token(field_id, false).as_str(),
                                convert_basic_token(field_id, true).as_str());
                            tab.hoist(&Token::StructDef{
                                id: Box::new(*field_id.clone()),
                                decl: Box::new(*field_type.clone()),
                            });
                        },
                        _ => {
                            wr.pub_field_decl(
                                convert_basic_token(field_id, false).as_str(),
                                convert_basic_token(field_type, true).as_str());
                        }
                    }
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
                Token::UnionDef{ref id, ref decl} => {
                    match **decl {
                        Token::Union{decl: ref decl, ref cases, ref default} => {
                            write_union(&ident, decl, cases, default, &mut tab, wr);
                        },
                        _ => { unreachable!() }
                    };
                },
                Token::VarOpaqueDecl{ref id, ref size} => {
                    wr.pub_field_decl(
                        convert_basic_token(id, false).as_str(), "Vec<u8>");
                },
                _ => {
                    println!("{:?}", field);
                    println!("UNIMPLEMENTED STRUCT FIELD");
                }
            };
        }
    });
    true
}

fn write_union(ident: &Token,
               ns_decl: &Box<Token>,
               cases: &Vec<Token>,
               default: &Box<Option<Token>>,
               tab: &mut CodegenState,
               wr: &mut CodeWriter) -> bool {

    let id = match *ident {
        Token::Ident(ref id) => { rustify(id) },
        _ => { return false }
    };
    let ns = match **ns_decl {
        Token::Decl{ref ty, ref id} => {
            ty
        }
        _=> { unreachable!() }
    };

    wr.pub_union_enum(id, |wr| {
        for arm in cases.iter() {
            match *arm {
                Token::UnionCase{ref vals, ref decl} => {
                    for case in vals.iter() {
                        let token =  tab.get_symbol(ns, &case);
                        if let Some(t) = token {
                            wr.enc_annotation(convert_basic_token(&t, false).as_str());
                        }
                        wr.enum_struct_decl(convert_basic_token(case, true).as_str(), |wr| {
                            match **decl {
                                Token::Decl{ty: ref field_type, id: ref field_id} => {
                                    match **field_type {
                                        Token::Union{decl: ref decl, ref cases, ref default} => {
                                            wr.field_decl(
                                                convert_basic_token(field_id, false).as_str(),
                                                convert_basic_token(field_id, true).as_str());
                                            tab.hoist(&Token::UnionDef{
                                                id: Box::new(*field_id.clone()),
                                                decl: Box::new(*field_type.clone()),
                                            });
                                        },
                                        Token::Struct(_) => {
                                            wr.field_decl(
                                                convert_basic_token(field_id, false).as_str(),
                                                convert_basic_token(field_id, true).as_str());
                                            tab.hoist(&Token::StructDef{
                                                id: Box::new(*field_id.clone()),
                                                decl: Box::new(*field_type.clone()),
                                            });
                                        },
                                        _ => {
                                            wr.field_decl(
                                                convert_basic_token(field_id, false).as_str(),
                                                convert_basic_token(field_type, true).as_str());
                                        }
                                    }
                                },
                                _ => { /* void decl probably */ }
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

fn write_enum(ident: &Token,
              fields: &Vec<(Token, Token)>,
              tab: &mut CodegenState,
              wr: &mut CodeWriter) -> bool {

    let id = match *ident {
        Token::Ident(ref id) => { rustify(id) },
        _ => { return false }
    };
    wr.xdr_enum(id.as_str(), |wr| {
        for &(ref field_id, ref field_val) in fields.iter() {
            match *field_val {
                Token::Blank => {
                    // Nothing to lookup here
                    wr.enum_decl(convert_basic_token(field_id, true).as_str(), "");
                }
                _ => {
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

fn write_typedef(def: &Box<Token>, wr: &mut CodeWriter) -> bool {
    match **def {
        Token::VarArrayDecl{ref ty, ref id, ref size} => {
            wr.pub_alias(convert_basic_token(&id, true), |wr| {
                wr.var_vec(convert_basic_token(&ty, true).as_str());
            });
        },
        Token::StringDecl{ref id, ref size} => {
            wr.pub_alias(convert_basic_token(&id, true), |wr| {
                // TODO Size this somehow. maybe make these &[u8]
                wr.write(String::from("String"));
            });
        },
        Token::Decl{ref ty, ref id} => {
            wr.pub_alias(convert_basic_token(&id, true), |wr| {
                wr.write(convert_basic_token(&ty, true).as_str());
            });
        },
        _ => {
            println!("UNIMPLEMENTED TYPEDEF");
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
                Token::Proc {ref return_type, ref name, ref arg_types, ref id} => {
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
                      ret_type: &Token, arg_types: &Vec<Token>,
                      wr: &mut CodeWriter) {
    let arg_names = (0..arg_types.len()).filter(|x| {
        match arg_types[*x] { Token::VoidDecl => { false}, _ => { true } }
    }).map(|x| { format!("arg{}", x) }).collect();
    let proc_name_str = convert_basic_token(proc_name, true);
    wr.match_option(&format!("{}RequestV{}::{}", prog_name, ver_num,
        proc_name_str.as_str()), &arg_names, |wr| {
        wr.write(&format!("self.{}_v{}(",
            convert_basic_token(proc_name, false).as_str().to_lowercase(),
            ver_num));
        wr.comma_fields(&arg_names);
        wr.raw_write(")");
        let has_return = match *ret_type {
            Token::VoidDecl => false,
            _ => true
        };
        wrap_proc_result(prog_name, ver_num, proc_name_str.as_str(),
            has_return, wr);
        wr.raw_write(".boxed()\n");
    });
}

fn write_service_version(prog_name: &String, ver_num: i64, procs: &Vec<Token>,
                         wr: &mut CodeWriter) {
    let version_fields = vec!["data"];
    wr.match_option(&format!("{}Request::V{}", prog_name, ver_num),
            &version_fields, |wr| {
        wr.let_match_block("res", "data", |wr| {
            for ptoken in procs {
                if let Token::Proc{ref return_type, ref name, ref arg_types, ref id} = *ptoken {
                    write_service_proc(prog_name,
                                       ver_num,
                                       (*name).as_ref(),
                                       &(**return_type),
                                       arg_types,
                                       wr);
                }
            }
            decoder_miss_future("procedure", wr);
        });
        wrap_version_result(prog_name, ver_num, wr);
    });
}

fn write_service(prog_name: &String, versions: &Vec<Token>,
                 wr: &mut CodeWriter) -> bool {
    let rust_prog_name = rustify(prog_name);
    wr.program_version_service(&format!("{}Service", rust_prog_name), |wr| {
        wr.alias("Request", |wr| {
            wr.raw_write(&format!("xdr_rpc::XdrRequest<{}Request>", rust_prog_name));
        });
        wr.alias("Response", |wr| {
            wr.raw_write(&format!("xdr_rpc::XdrResponse<{}Response>", rust_prog_name));
        });
        wr.alias("Error", |wr| {
            wr.raw_write("io::Error");
        });
        wr.alias("Future", |wr| {
            wr.raw_write("BoxFuture<Self::Response, Self::Error>");
        });
        wr.dispatch_function(|wr| {
            wr.write_line("let xid = req.xid;");
            wr.match_block("req.val", |wr| {
                for vtoken in versions {
                    if let Token::Version{ref name, ref id, ref procs} = *vtoken {
                        if let Token::Constant(id_num) = **id {
                            write_service_version(&rust_prog_name, id_num,
                                                  procs, wr);
                        }
                    }
                }
                decoder_miss_future("version", wr);
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

fn write_codec(prog_name: &String, versions: &Vec<Token>, wr: &mut CodeWriter) -> bool {
    let prog_name = rustify(prog_name);

    // TODO more rigourous way of getting these vals
    let app_codec = prog_name.replace("Prog", "AppCodec");
    let codec     = prog_name.replace("Prog", "Codec");
    let protocol  = prog_name.replace("Prog", "Protocol");
    let app_req   = prog_name.replace("Prog", "ProgRequest");
    let app_res   = prog_name.replace("Prog", "ProgResponse");

    wr.pub_struct(app_codec.clone(), |wr| {});
    wr.impl_("", "Codec", app_codec.as_str(), |wr| {
        wr.alias_impl("", "In", |wr| {
            wr.write(app_req.as_str());
        });
        wr.alias_impl("", "Out", |wr| {
            wr.write(app_res.as_str());
        });
        codec_fns(prog_name.as_str(), wr);
    });

    wr.impl_("", "app_codec", app_codec.as_str(), |wr| {
        app_codec_fn(prog_name.as_str(), wr);
    });

    wr.pub_alias(codec.as_str(), |wr| {
        wr.write(format!("XdrCodec<{}>", app_codec.as_str()));
    });

    wr.pub_struct(protocol.clone(), |wr| {});
    wr.impl_("<T: Io + 'static>", "ServerProto<T>", protocol.as_str(), |wr| {
        wr.alias_impl("", "Request", |wr| {
            wr.write(format!("xdr_rpc::XdrRequest<{}>", app_req.as_str()));
        });
        wr.alias_impl("", "Response", |wr| {
            wr.write(format!("xdr_rpc::XdrResponse<{}>", app_res.as_str()));
        });
        wr.alias_impl("", "Transport", |wr| {
            wr.write(format!("Framed<T, {}>", codec.as_str()));
        });
        wr.alias_impl("", "BindTransport", |wr| {
            wr.write("io::Result<Self::Transport>");
        });
        proto_fn(codec.as_str(), app_codec.as_str(), wr);
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
                                  prog_name.to_lowercase(),
                                  ver_num,
                                  convert_basic_token(proc_name, false).as_str().to_lowercase());

    proc_decoder(rustify(prog_name).as_str(), &proc_decoder_fn, ver_num, wr, |wr| {
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
    version_decoder(rustify(prog_name).as_str(), &version_decoder_fn, wr, |wr| {
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
    prog_decoder(rustify(prog_name).as_str(), &prog_decoder_fn, wr, |wr| {
        wr.match_block("version", |wr| {
            for vtoken in versions {
                if let Token::Version{ref name, ref id, ref procs} = *vtoken {
                    if let Token::Constant(id_num) = **id {
                        wr.match_option(
                            &format!("{}u32", id_num),
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
                    let has_return = match **return_type {
                        Token::VoidDecl => false,
                        _ => true
                    };
                    encoder_proc(prog_name,
                                 convert_basic_token(name, true).as_str(),
                                 ver_num, has_return, wr);
                }
            }
            decoder_miss("procedure", wr);
        });
        wr.write_line("Ok(())");
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

//fn write_codec(name: &String, progs: &Vec<Token>, wr: &mut CodeWriter) -> bool {
//   // wr.namespace(name, |wr| {
//   //     wr.write_line("use super::*;");
//        for ptoken in progs {
//            if let Token::Program{ref name, ref id, ref versions} = *ptoken {
//                if let Token::Ident(ref name_str) = **name{
//                    if let Token::Constant(id_num) = **id {
//                        let prog_decoder_fn = format!("{}_decode", name_str.to_lowercase());
//                        top_decoder(rustify(name_str).as_str(), wr, |wr| {
//                            wr.match_block("header.program", |wr| {
//                                wr.match_option(&format!("{}u32", id_num), &Vec::<String>::new(), |wr| {
//                                    prog_decoder_call(&prog_decoder_fn, wr);
//                                });
//                                decoder_miss("program", wr);
//                            });
//                        });
//                    }
//                    ()
//                }
//            }
//        }
//   // });
//
//    true
//}

// fn write_namespace(name: &String, progs: &Vec<Token>, wr: &mut CodeWriter) -> bool {
//     wr.namespace(name, |wr| {
//         wr.write_line("use super::*;");
//         for ptoken in progs {
//             if let Token::Program{ref name, ref id, ref versions} = *ptoken {
//                 if let Token::Ident(ref name_str) = **name{
//                     write_program(name_str, &versions, wr);
//                     write_service(name_str, &versions, wr);
//                     if let Token::Constant(id_num) = **id {
//                         write_decoder(name_str, id_num, versions, wr);
//                         write_encoder(name_str, versions, wr);
//                     }
//                     ()
//                 }
//             }
//         }
//     });
//     true
// }

#[derive(Debug)]
struct CodegenState<'a> {
    //table: HashMap<&'a str, User<'a>>,
    name: &'a str,
    table: HashMap<(Token, Token), Token>,
    hoister: Vec<Token>,
}

impl<'a> CodegenState<'a> {
    fn new(name: &str) -> CodegenState {
        let table = HashMap::new();
        let hoister = Vec::new();
        CodegenState {
            name: name,
            table: table,
            hoister: hoister,
        }
    }

    fn hoist(&mut self, tkn: &Token) {
        self.hoister.push(tkn.clone())
    }

    fn get_hoisted(&mut self) -> Vec<Token> {
        self.hoister.clone()
    }

    fn add_symbol(&mut self, ns_tkn: &Token, id_tkn: &Token, val: &Token) -> Option<&Token> {
        let key = (ns_tkn.clone(), id_tkn.clone());
        if self.table.contains_key(&key) {
            self.table.get(&key)
        } else {
            self.table.insert(key, val.clone());
            None
        }
    }

    fn get_symbol(&mut self, ns_tkn: &Token, id_tkn: &Token) -> Option<Token> {
        let key = (ns_tkn.clone(), id_tkn.clone());
        Some(self.table.get(&key).unwrap().clone())
    }
}



//    codegen(wr, tokens, &mut tab);
//    codegen(wr, Some(tab.get_hoisted()), &mut tab)

pub struct CodeGen<'a: 'b, 'b: 'c, 'c> {
	types_wr: &'c mut CodeWriter<'a>,
	codec_wr: &'c mut CodeWriter<'b>,
    service_wr: &'c mut CodeWriter<'c>,
    tokens: Option<Vec<Token>>,
    state: CodegenState<'c>,
}

impl<'a, 'b, 'c> CodeGen<'a, 'b, 'c> {
    pub fn new(tw: &'c mut CodeWriter<'a>,
			   cw: &'c mut CodeWriter<'b>,
			   sw: &'c mut CodeWriter<'c>) -> CodeGen<'a, 'b, 'c> {
        CodeGen {
			types_wr: tw,
			codec_wr: cw,
			service_wr: sw,
            tokens: None,
            state: CodegenState::new("CodegenTable"),
		}
    }

    pub fn compile(&mut self, source: String, dump_parse: bool) -> Result<&'static str, ()> {
        let bytes = source.into_bytes();
        let not_yet_parsed = bytes.as_slice();
        if dump_parse {
            parser::parse(not_yet_parsed, dump_parse);
            Ok("Dumped parse tree")
        } else {
            self.tokens = parser::parse(not_yet_parsed, dump_parse);
            self.codec_wr.write_codec_header();
            self.types_wr.write_proto_header();
            self.service_wr.write_service_header();

            self.codegen_all();

            //while !self.state.hoister.is_empty() {
            //    self.codegen_all();
            //}
            Ok("Complete codegen")
        }
    }

    fn codegen_all(&mut self) -> Result<&'static str, ()> {
        for token in self.tokens.as_ref().unwrap() {
            match *token {
                // These three tokens are useless to us, just ignore them
                Token::Blank => {},
                Token::Comment(_) => {},
                Token::CodeSnippet(_) => {}, // TODO maybe do something special with this

                // These tokens are incredibly useful
                Token::UnionDef{ref id, ref decl} => {
                    match **decl {
                        Token::Union{decl: ref decl, ref cases, ref default} => {
                            write_union(&*id, decl, cases, default, &mut self.state, self.types_wr);
                        },
                        _ => { unreachable!() }
                    };
                },
                Token::StructDef{ref id, ref decl} => {
                    match **decl {
                        Token::Struct(ref fields) => {
                            write_struct(id, fields, &mut self.state, self.types_wr);
                        },
                        _ => { unreachable!() }
                    };
                },
                Token::EnumDef{ref id, ref decl} => {
                    match **decl {
                        Token::Enum(ref fields) => {
                            write_enum(id, fields, &mut self.state, self.types_wr);
                        },
                        _ => { unreachable!() }
                    };
                },
                Token::TypeDef(ref def) => {
                    write_typedef(def, self.types_wr);
                },
                Token::Program{ref name, ref id, ref versions} => {
                    if let Token::Ident(ref name_str) = **name{
                        write_codec(name_str, &versions, self.codec_wr);
                        write_program(name_str, &versions, self.types_wr);
                        write_service(name_str, &versions, self.service_wr);
                        if let Token::Constant(id_num) = **id {
                            write_decoder(name_str, id_num, versions, self.types_wr);
                            write_encoder(name_str, versions, self.types_wr);
                        };
                    }
                },

                //Token::Namespace{ref name, ref progs} => {
                //    match **name {
                //        Token::Ident(ref s) => {
                //            write_namespace(s, &progs, self.service_wr);
                //        },
                //        _ => { unreachable!() }
                //    }
                //}
                _ => {
                    println!("Codegen isn't supported for this token yet");
                    break;
                }
            }
        }
        Ok("Complete codegen")
    }
}
