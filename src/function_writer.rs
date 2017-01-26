use code_writer::CodeWriter;

pub fn top_decoder<S: AsRef<str>, F>(prog_name: S, wr: &mut CodeWriter, cb: F)
            where F : Fn(&mut CodeWriter) {
    wr.expr_block(
            &format!("pub fn decode(buf: &mut EasyBuf) -> io::Result<Option<{}Request>>",
            prog_name.as_ref()), "", |wr| {
        wr.write(
r###"let header_res = serde_xdr::from_bytes::<xdr_rpc::XdrRpcHeader>(buf.as_slice());
    let header = match header_res {
        Ok((h, consumed)) => {
            buf.drain_to(consumed);
            h
        },
        Err(e) => {
            match e {
                serde_xdr::EncoderError::Io(i) => {
                    return Err(i);
                },
                serde_xdr::EncoderError::Unknown(s) => {
                    return Err(io::Error::new(io::ErrorKind::Other,
                        format!("failed to read header: {}", s)));
                }
            }
        }
    };

    match header.rpc_vers {
        2u32 => {},
        _ => {
            return Err(io::Error::new(io::ErrorKind::Other, "unknown RPC version"));
        }
    }
"###);

        cb(wr);
    });
}

pub fn decoder_miss_impl<S: AsRef<str>>(type_name: &str, suffix: &str, s: S,
                                        wr: &mut CodeWriter) {
    wr.match_option("_", &Vec::<String>::new(), |wr| {
        wr.write_line(&format!(
            "return {}(io::Error::new(io::ErrorKind::Other, \"unknown {}\")){};",
            type_name, s.as_ref(), suffix));
    });
}

pub fn decoder_miss<S: AsRef<str>>(s: S, wr: &mut CodeWriter) {
    decoder_miss_impl("Err", "", s, wr);
}

pub fn decoder_miss_future<S: AsRef<str>>(s: S, wr: &mut CodeWriter) {
    decoder_miss_impl("future::err", ".boxed()", s, wr);
}

pub fn prog_decoder<S1: AsRef<str>, S2: AsRef<str>, F>(prog_name:S1,
                                                       fn_name: S2,
                                                       wr: &mut CodeWriter,
                                                       cb: F)
        where F : Fn(&mut CodeWriter) {
    wr.expr_block(&format!(
r###"pub fn {}(program: u32, version: u32, procedure: u32, buf: &mut EasyBuf) ->
    io::Result<Option<{}Request>>"###, fn_name.as_ref(), prog_name.as_ref()),
    "", cb);
}

pub fn prog_decoder_call<S: AsRef<str>>(fn_name: S, wr: &mut CodeWriter) {
    wr.write_line(&format!("{}(header.version, header.procedure, buf)",
        fn_name.as_ref()));
}

pub fn version_decoder_match<F>(wr: &mut CodeWriter, cb: F)
        where F : Fn(&mut CodeWriter) {
    wr.let_match_block("request", "procedure", cb);
}

pub fn version_decoder<S1: AsRef<str>, S2: AsRef<str>, F>(prog_name: S1,
                                                          fn_name: S2,
                                                          wr: &mut CodeWriter,
                                                          cb: F)
        where F : Fn(&mut CodeWriter) {
    wr.expr_block(&format!(
r###"pub fn {}(procedure: u32, buf: &mut EasyBuf) ->
    io::Result<Option<{}Request>>"###, fn_name.as_ref(), prog_name.as_ref()),
    "", cb);
}

pub fn version_decoder_call<S: AsRef<str>>(fn_name: S, wr: &mut CodeWriter) {
    wr.write_line(&format!("{}(procedure, buf)", fn_name.as_ref()));
}

pub fn version_decoder_finalize<S: AsRef<str>>(prog_name: S, ver_num: i64,
                                               wr: &mut CodeWriter) {
    wr.write_line(&format!("Ok(Some({}Request::V{}(request.unwrap().unwrap())))",
        prog_name.as_ref(), ver_num));
}

pub fn proc_decoder<S1: AsRef<str>, S2: AsRef<str>, F>(prog_name: S1,
                                                       fn_name: S2,
                                                       ver_num: i64,
                                                       wr: &mut CodeWriter,
                                                       cb: F)
        where F : Fn(&mut CodeWriter) {
    wr.expr_block(
        &format!("pub fn {}(buf: &mut EasyBuf) -> io::Result<Option<{}RequestV{}>>",
        fn_name.as_ref(), prog_name.as_ref(), ver_num), "", cb);
}

pub fn proc_decoder_call<S: AsRef<str>>(fn_name: S, wr: &mut CodeWriter) {
    wr.write_line(&format!("{}(buf)", fn_name.as_ref()));
}

pub fn proc_arg_decoder<S: AsRef<str>>(arg_index: u32, arg_type: S,
                                       wr: &mut CodeWriter) {
    wr.write_line(&format!(
r###"let res{0} = serde_xdr::from_bytes::<{1}>(buf.as_slice());
    let arg{0} = match res{0} {{
        Ok((arg, consumed)) => {{
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
                        format!("argument {0} parse failure: {{}}", s)));
                }}
            }}
        }}
    }};
"###, arg_index, arg_type.as_ref()));
}

pub fn proc_decoder_finalize<S1: AsRef<str>, S2: AsRef<str>>(
        req_type: S1, req_name: S2, n_args: u32, wr: &mut CodeWriter) {
    wr.write(&format!("Ok(Some({}::{}", req_type.as_ref(), req_name.as_ref()));

    let arg_list = (0..n_args).map(|x| { format!("arg{}", x) }).collect();
    wr.optional_fields(&arg_list);
    wr.raw_write("))\n");
}

pub fn encoder<S: AsRef<str>, F>(prog_name: S, wr: &mut CodeWriter, cb: F)
        where F : Fn(&mut CodeWriter) {
    wr.expr_block(&format!(
        "pub fn encode(msg: {}Response, buf: &mut Vec<u8>) -> io::Result<()>",
        prog_name.as_ref()), "", cb);
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
                                                    has_return: bool,
                                                    wr: &mut CodeWriter) {
    let mut arg_list = if has_return {
        vec!["r"]
    } else {
        Vec::<&str>::new()
    };

    wr.match_option(&format!("{}ResponseV{}::{}", prog_name.as_ref(), ver_num,
            proc_name.as_ref()), &arg_list, |wr| {
        if has_return {
            wr.write_line("try!(serde_xdr::to_bytes(&r, buf));");
        }
    });
}

pub fn wrap_proc_result<S1: AsRef<str>, S2: AsRef<str>>(prog_name: S1,
                                                        ver_num: i64,
                                                        proc_name: S2,
                                                        has_return: bool,
                                                        wr: &mut CodeWriter) {
    let wrapper_arg = if has_return {
        "(r)"
    } else {
        ""
    };
    wr.raw_write(
&format!(r###".map(|r| {{
                {}ResponseV{}::{}{}
              }})"###,
    prog_name.as_ref(), ver_num, proc_name.as_ref(), wrapper_arg));
}

pub fn wrap_version_result<S: AsRef<str>>(prog_name: S, ver_num: i64,
                                          wr: &mut CodeWriter) {
    wr.write_line(
&format!(r###"res.map(|r| {{
            {}Response::V{}(r)
          }}).boxed()"###, prog_name.as_ref(), ver_num));
}
