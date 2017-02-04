use std::io::Write;
use std::fmt::Debug;

pub struct CodeWriter<'a> {
    writer: &'a mut (Write + 'a),
    indent: String,
}

impl<'a> CodeWriter<'a> {
    pub fn new(writer: &'a mut Write) -> CodeWriter<'a> {
        CodeWriter {
            writer: writer,
            indent: "".to_string(), // Two space master race
        }
    }

    pub fn same_line<F>(&mut self, mut cb: F) where F : FnMut(&mut CodeWriter) {
        cb(&mut CodeWriter {
            writer: self.writer,
            indent: format!(""),
        });
    }

    pub fn indented<F>(&mut self, mut cb: F) where F : FnMut(&mut CodeWriter) {
        cb(&mut CodeWriter {
            writer: self.writer,
            indent: format!("{}  ", self.indent),
        });
    }

    pub fn comment(&mut self, comment: &str) {
        if comment.is_empty() {
            self.write_line("//");
        } else {
            self.write_line(&format!("// {}", comment));
        }
    }

    pub fn write<S : AsRef<str>>(&mut self, line: S) {
        let s: String = [self.indent.as_ref(), line.as_ref()].concat();
        let _ = self.writer.write_all(s.as_bytes());
    }

    pub fn raw_write<S : AsRef<str>>(&mut self, text: S) {
        let _ = self.writer.write_all(text.as_ref().to_string().as_bytes());
    }

    pub fn write_line<S : AsRef<str>>(&mut self, line: S) {
        (if line.as_ref().is_empty() {
            self.writer.write_all("\n".as_bytes())
        } else {
            let s: String = [self.indent.as_ref(), line.as_ref(), "\n"].concat();
            self.writer.write_all(s.as_bytes())
        }).unwrap();
    }

    pub fn write_types_header(&mut self) {
        self.comment("Autogenerated by xdrust");
        self.write_line("#[allow(dead_code)]");
        self.write_line("use std::{io, fmt};");
        self.write_line("use serde_xdr;");
        self.write_line("");
    }

    pub fn write_proto_header(&mut self) {
        self.comment("autogenerated by xdrust");
        self.comment("translated XDR->Rust types and functions");
        self.write_line("#[allow(dead_code)]");
        self.write_line("use std::{io, fmt};");
        self.write_line("use serde_xdr;");
        self.write_line("use xdr_rpc;");
        self.write_line("use tokio_core::io::EasyBuf;");
        self.write_line("");
    }

    fn alias_impl<S : AsRef<str>, F>(&mut self, prefix: &str,  name: S, mut cb: F)
        where F : FnMut(&mut CodeWriter) {
            self.write(&format!("{}type {} = ", prefix, name.as_ref()));
            cb(self);
            self.raw_write(";\n")
    }

    pub fn alias<S : AsRef<str>, F>(&mut self, name: S, mut cb: F)
            where F : FnMut(&mut CodeWriter) {
        self.alias_impl("", name, cb);
    }

    pub fn pub_alias<S : AsRef<str>, F>(&mut self, name: S, mut cb: F)
            where F : FnMut(&mut CodeWriter) {
        self.alias_impl("pub ", name, cb);
    }

    pub fn pub_union_enum<S : AsRef<str>, F>(&mut self, name: S, mut cb: F) where F : FnMut(&mut CodeWriter) {
        self.write_line("");
        self.write_line("#[derive(Serialize, Deserialize, PartialEq, Debug)]");
        self.write_line("#[serde(rename(deserialize = \"__UNION_SYMBOL__\"))]");
        self.expr_block(&format!("pub enum {}", name.as_ref()), "", cb);
    }

    pub fn pub_enum<S : AsRef<str>, F>(&mut self, name: S, mut cb: F) where F : FnMut(&mut CodeWriter) {
        self.write_line("");
        self.write_line("#[derive(Serialize, Deserialize, PartialEq, Debug)]");
        self.expr_block(&format!("pub enum {}", name.as_ref()), "", cb);
    }

    pub fn pub_struct<S : AsRef<str>, F>(&mut self, name: S, mut cb: F)
        where F : FnMut(&mut CodeWriter) {
            self.write_line("");
            self.write_line("#[derive(Serialize, Deserialize, PartialEq, Debug)]");
            self.expr_block(&format!("pub struct {}", name.as_ref()), "", cb);
    }


    pub fn program_version_request<S: AsRef<str>, F>(&mut self, prog_name: S,
                                                     ver_num: i64, mut cb: F)
            where F: FnMut(&mut CodeWriter) {
        self.pub_enum(&format!("{}RequestV{}", prog_name.as_ref(), ver_num),
            cb);

    }

    pub fn comma_fields<S: AsRef<str>>(&mut self, fields: &Vec<S>) {
        for (i, arg) in fields.iter().enumerate() {
            if i > 0 {
                self.raw_write(", ");
            }
            self.raw_write(&format!("{}", arg.as_ref()));
        }
    }

    pub fn optional_fields<S: AsRef<str>>(&mut self, fields: &Vec<S>) {
        if fields.len() == 0 {
            return;
        }

        self.raw_write("(");
        self.comma_fields(fields);
        self.raw_write(")");
    }

    pub fn version_proc_request<S1: AsRef<str>, S2: AsRef<str>>(&mut self,
                                                            name: S1,
                                                            args: &Vec<S2>) {
        self.write(name);
        self.optional_fields(args);
        self.raw_write(",\n");
    }

    pub fn program_version_response<S: AsRef<str>, F>(&mut self, prog_name: S,
                                                     ver_num: i64, mut cb: F)
            where F: FnMut(&mut CodeWriter) {
        self.pub_enum(&format!("{}ResponseV{}", prog_name.as_ref(), ver_num),
            cb);
    }

    pub fn version_proc_response<S, Q>(&mut self, name: S, ret: Option<Q>)
                                        where S: AsRef<str>, Q: AsRef<str> + Debug {
        self.write(name);
        if let Some(s) = ret {
            self.raw_write(&format!("({})", s.as_ref()));
        }
        self.raw_write(",\n");
    }

    pub fn program_version_service<S: AsRef<str>, F>(&mut self, service_name: S,
                                                  cb: F)
            where F: FnMut(&mut CodeWriter) {
        self.expr_block(&format!("impl Service for {}", service_name.as_ref()),
            "", cb);
    }

    pub fn dispatch_function<F>(&mut self, mut cb: F)
            where F: FnMut(&mut CodeWriter) {
        self.expr_block("fn call (&self, req: Self::Request) -> Self::Future",
            "", cb);
    }

    pub fn match_block<S: AsRef<str>, F>(&mut self, s: S, mut cb: F)
            where F: FnMut(&mut CodeWriter) {
        self.expr_block(&format!("match {} ", s.as_ref()), "", cb);
    }

    pub fn let_match_block<S1: AsRef<str>, S2: AsRef<str>, F>(&mut self,
                                                              assign: S1,
                                                              value: S2,
                                                              cb: F)
            where F: FnMut(&mut CodeWriter) {
        self.expr_block(&format!("let {} = match {}", assign.as_ref(),
            value.as_ref()), ";", cb);
    }

    pub fn match_option<S1: AsRef<str>, S2: AsRef<str>, F>(&mut self,
                                                            type_name: S1,
                                                            fields: &Vec<S2>,
                                                            cb: F)
            where F: FnMut(&mut CodeWriter) {
        self.write(type_name);
        self.optional_fields(fields);
        self.raw_write(" => {\n");
        self.indented(cb);
        self.write_line("},");
    }

    pub fn namespace<S: AsRef<str>, F>(&mut self, name: S, mut cb: F)
            where F: FnMut(&mut CodeWriter) {
        self.expr_block(&format!("pub mod {}", name.as_ref()), "", cb)
    }

    pub fn var_vec(&mut self, type_: &str) {
        self.write(&format!("Vec<{}>", type_));
    }

    pub fn enum_tuple_decl<F>(&mut self, name: &str, mut cb: F)
        where F : FnMut(&mut CodeWriter) {
            self.write(&format!("{}(", name));
            cb(self);
            self.raw_write("),\n");
    }

    pub fn enum_struct_decl<F>(&mut self, name: &str, mut cb: F)
        where F : FnMut(&mut CodeWriter) {
            self.write_line(&format!("{} {{", name));
            self.indented(cb);
            self.write_line("},");
    }

    pub fn enum_decl(&mut self, name: &str, val: &str) {
        // TODO is Option<&str> cleaner?
        if val == "" {
            self.write_line(&format!("{},", name));
        } else {
            self.write_line(&format!("{} = {},", name, val));
        }
    }

    // TODO generalized version should replace non _fn impl
    pub fn pub_field_decl_fn<F>(&mut self, name: &str, mut cb: F)
        where F : FnMut(&mut CodeWriter) {
        self.write(&format!("pub {}: ", name));
        self.same_line(cb);
        self.raw_write(",\n");
    }

    pub fn pub_field_decl(&mut self, name: &str, field_type: &str) {
        self.write_line(&format!("pub {}: {},", name, field_type));
    }

    pub fn enc_annotation(&mut self, name: &str) {
        self.write_line(&format!("#[serde(rename = \"{}\")]", name));
    }

    pub fn field_decl(&mut self, name: &str, field_type: &str) {
        self.write_line(&format!("{}: {},", name, field_type));
    }

    pub fn expr_block<F>(&mut self, prefix: &str, trailing_char: &str, mut cb: F)
        where F : FnMut(&mut CodeWriter) {
            self.block(&format!("{} {{", prefix),
                &format!("}}{}", trailing_char), cb);
    }

    pub fn xdr_enum<F>(&mut self, prefix: &str, mut cb: F) where F : FnMut(&mut CodeWriter) {
            self.block(&format!("xdr_enum!({} {{", prefix), "});", cb);
    }

    pub fn block<F>(&mut self, first_line: &str, last_line: &str, mut cb: F)
        where F : FnMut(&mut CodeWriter) {
            self.write_line(first_line);
            self.indented(cb);
            self.write_line(last_line);
    }

}
