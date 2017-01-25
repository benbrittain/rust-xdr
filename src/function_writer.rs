use code_writer::CodeWriter;

pub fn top_decoder<S: AsRef<str>, F>(prog_name: S, wr: &mut CodeWriter, cb: F)
            where F : Fn(&mut CodeWriter) {
    wr.expr_block(
            &format!("fn decode(buf: &[u8]) -> io::Result<Option<{}Request>>",
            prog_name.as_ref()), false, |wr| {
        wr.write(
r###"let header_res = serde_xdr::from_bytes::<XdrRpcHeader>(buf.to_slice());
    let header = match header_res {
        Ok(h, consumed) => {
            buf.drain_to(consumed);
            h
        },
        Err(e) => {
            match e {
                serde_xdr::EncoderError::Io(i) => {
                    return Err(i);
                },
                serde_xdr::EncoderError::Unknown(s) => {
                    return io::Error::new(io::ErrorKind::Other,
                        format!("failed to read header: {}", s));
                }
            }
        }
    };

    match header.rpc_vers {
        2u32 => {},
        _ => {
            return io::Error:new(io::ErrorKind::Other, "unknown RPC version");
        }
    }
"###);

        cb(wr);
    });
}

pub fn decoder_miss<S: AsRef<str>>(s: S, wr: &mut CodeWriter) {
    wr.match_option("_", &Vec::<String>::new(), |wr| {
        wr.write_line(&format!(
            "io::Error::new(io::ErrorKind::Other, \"unknown {}\");",
            s.as_ref()));
    });
}

pub fn prog_decoder<S1: AsRef<str>, S2: AsRef<str>, F>(prog_name:S1,
                                                       fn_name: S2,
                                                       wr: &mut CodeWriter,
                                                       cb: F)
        where F : Fn(&mut CodeWriter) {
    wr.expr_block(&format!(
r###"pub fn {}(version: u32, procedure: u32, buf: &[u8]) ->
    io::Result<Option<{}Request>>"###, fn_name.as_ref(), prog_name.as_ref()),
    false, cb);
}

pub fn prog_decoder_call<S: AsRef<str>>(fn_name: S, wr: &mut CodeWriter) {
    wr.write_line(&format!("{}(header.version, header.procedure, buf);",
        fn_name.as_ref()));
}

pub fn version_decoder_match<F>(wr: &mut CodeWriter, cb: F)
        where F : Fn(&mut CodeWriter) {
    wr.expr_block("let request = match procedure", false, cb);
    wr.write_line(";");
}

pub fn version_decoder<S1: AsRef<str>, S2: AsRef<str>, F>(prog_name: S1,
                                                          fn_name: S2,
                                                          wr: &mut CodeWriter,
                                                          cb: F)
        where F : Fn(&mut CodeWriter) {
    wr.expr_block(&format!(
r###"pub fn {}(procedure: u32, buf: &[u8]) ->
    io::Result<Option<{}Request>>"###, fn_name.as_ref(), prog_name.as_ref()),
        false, cb);
}

pub fn version_decoder_call<S: AsRef<str>>(fn_name: S, wr: &mut CodeWriter) {
    wr.write_line(&format!("{}(procedure, buf);", fn_name.as_ref()));
}

pub fn version_decoder_finalize<S: AsRef<str>>(prog_name: S, ver_num: i64,
                                               wr: &mut CodeWriter) {
    wr.write_line(&format!("Ok(Some({}Request::V{}(request)));",
        prog_name.as_ref(), ver_num));
}

pub fn proc_decoder<S1: AsRef<str>, S2: AsRef<str>, F>(prog_name: S1,
                                                       fn_name: S2,
                                                       wr: &mut CodeWriter,
                                                       cb: F)
        where F : Fn(&mut CodeWriter) {
    wr.expr_block(
        &format!("pub fn {}(buf: &[u8]) -> io::Result<Option<{}Request>>",
        fn_name.as_ref(), prog_name.as_ref()), false, cb);
}

pub fn proc_decoder_call<S: AsRef<str>>(fn_name: S, wr: &mut CodeWriter) {
    wr.write_line(&format!("{}(buf);", fn_name.as_ref()));
}

pub fn proc_arg_decoder<S: AsRef<str>>(arg_index: u32, arg_type: S,
                                       wr: &mut CodeWriter) {
    wr.write_line(&format!(
r###"let res{0} = serde_xdr::from_bytes::<{1}>(buf.to_slice());
    let arg{0} = match res{0} {{
        Some(arg, consumed) => {{
            buf.drain_to(consumed);
            arg
        }},
        Err(e) => {{
            match e {{
                serde_xdr::EncoderError::Io(i) => {{
                    return Err(i);
                }},
                serde_xdr::EncoderError::Unknown(s) => {{
                    return Err(io::Error::new(io::ErrorKind::Other,
                        format!("argument {0} parse failure: {{}}"), s));
                }}
            }}
        }}
    }};
"###, arg_index, arg_type.as_ref()));
}

pub fn proc_decoder_finalize<S1: AsRef<str>, S2: AsRef<str>>(
        req_type: S1, req_name: S2, n_args: u32, wr: &mut CodeWriter) {
    wr.write(&format!("{}::{}(", req_type.as_ref(), req_name.as_ref()));

    let arg_list = (0..n_args).map(|x| { format!("arg{}", x) }).collect();
    wr.comma_fields(&arg_list);
    wr.raw_write(");\n");
}

pub fn encoder<S: AsRef<str>, F>(prog_name: S, wr: &mut CodeWriter, cb: F)
        where F : Fn(&mut CodeWriter) {
    wr.expr_block(&format!(
        "pub fn encode(msg: {}Response, buf: &mut Vec<u8>) -> io::Result<()>",
        prog_name.as_ref()), false, cb);
}

pub fn encoder_version<S: AsRef<str>, F>(prog_name: S, ver_num: i64,
                                         wr: &mut CodeWriter, cb:F)
        where F : Fn(&mut CodeWriter) {
    wr.match_option(&format!("{}Response::V{}", prog_name.as_ref(), ver_num),
        &vec!["rsp"], cb);
}

pub fn encoder_proc<S1: AsRef<str>, S2: AsRef<str>>(prog_name: S1,
                                                    proc_name: S2,
                                                    ver_num: i64,
                                                    wr: &mut CodeWriter) {
    wr.match_option(&format!("{}ResponseV{}::{}", prog_name.as_ref(), ver_num,
            proc_name.as_ref()), &vec!["r"], |wr| {
        wr.write_line("try!(serde_xdr::to_bytes(r, buf));");
    });
}
