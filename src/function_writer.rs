use code_writer::CodeWriter;

pub fn top_decoder<F>(wr: &mut CodeWriter, cb: F)
            where F : Fn(&mut CodeWriter) {
    wr.expr_block(
        "fn decode(buf: &mut EasyBuf) -> io::Result<Option<Self::In>>", false, |wr| {
        wr.write(
r###"let header_res = serde_xdr::from_bytes<XdrRpcHeader>(buf.to_slice());
    let header = match header_res {
        Ok(h, consumed) => {
            buf.drain_to(consumed);
            h
        },
        Err(e) => {
            match e => {
                Io(i) => {
                    return Err(i);
                },
                Other(s) => {
                    return io::Error::new(io::ErrorKind::Other,
                        format!("failed to read header: {}", s));
                }
            }
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

pub fn prog_decoder<S: AsRef<str>, F>(fn_name: S, wr: &mut CodeWriter, cb: F)
        where F : Fn(&mut CodeWriter) {
    wr.expr_block(&format!(
r###"pub fn {}(version: u32, proc: u32, buf: &mut EasyBuf) ->
    io::Result<Option<Self::In>>"###, fn_name.as_ref()), false, cb);
}

pub fn prog_decoder_call<S: AsRef<str>>(fn_name: S, wr: &mut CodeWriter) {
    wr.write_line(&format!("{}(header.version, header.proc, buf);",
        fn_name.as_ref()));
}

pub fn version_decoder_match<F>(wr: &mut CodeWriter, cb: F)
        where F : Fn(&mut CodeWriter) {
    wr.expr_block("let request = match proc", false, cb);
}

pub fn version_decoder<S: AsRef<str>, F>(fn_name: S, wr: &mut CodeWriter, cb: F)
        where F : Fn(&mut CodeWriter) {
    wr.expr_block(&format!(
r###"pub fn {}(proc: u32, buf: &mut EasyBuf) ->
    io::Result<Option<Self::In>>"###, fn_name.as_ref()), false, cb);
}

pub fn version_decoder_call<S: AsRef<str>>(fn_name: S, wr: &mut CodeWriter) {
    wr.write_line(&format!("{}(proc, buf);", fn_name.as_ref()));
}

pub fn version_decoder_finalize<S: AsRef<str>>(prog_name: S, ver_num: i64,
                                               wr: &mut CodeWriter) {
    wr.write_line(&format!("Ok(Some({}Request::V{}(request)));",
        prog_name.as_ref(), ver_num));
}

pub fn proc_decoder<S: AsRef<str>, F>(fn_name: S, wr: &mut CodeWriter, cb: F)
        where F : Fn(&mut CodeWriter) {
    wr.expr_block(&format!(
r###"pub fn {}(buf: &mut EasyBuf) ->
    io::Result<Option<Self::In>>"###, fn_name.as_ref()), false, cb);
}

pub fn proc_decoder_call<S: AsRef<str>>(fn_name: S, wr: &mut CodeWriter) {
    wr.write_line(&format!("{}(buf);", fn_name.as_ref()));
}

pub fn proc_arg_decoder<S: AsRef<str>>(arg_index: u32, arg_type: S,
                                       wr: &mut CodeWriter) {
    wr.write_line(&format!(
r###"let res{0} = serde_xdr::from_bytes<{1}>(buf.to_slice());
    let arg{0} = match res{0} {{
        Some(arg, consumed) => {{
            buf.drain_to(consumed);
            arg
        }},
        Err(e) => {{
            match e => {{
                Io(i) => {{
                    return Err(i);
                }},
                Other(s) => {{
                    return Err(io::Error::new(io::ErrorKind::Other,
                        format!("argument {0} parse failure: {}"), s));
                }}
            }}
        }}
    }}
"###, arg_index, arg_type.as_ref()));
}

pub fn proc_decoder_finalize<S1: AsRef<str>, S2: AsRef<str>>(
        req_type: S1, req_name: S2, n_args: u32, wr: &mut CodeWriter) {
    wr.write(&format!("{}::{}(", req_type.as_ref(), req_name.as_ref()));

    let arg_list = (0..n_args).map(|x| { format!("arg{}", x) }).collect();
    wr.comma_fields(&arg_list);
    wr.raw_write(");\n");
}
