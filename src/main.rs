#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate nom;
#[macro_use]
extern crate serde_xdr;

extern crate rustc_serialize;
extern crate serde;
extern crate clap;

use std::env;
use std::fs::File;
use std::str;
use std::io::{self, Write, Read};
use clap::*;

mod parser;
mod codegen;
mod code_writer;
mod function_writer;

use code_writer::CodeWriter;
use codegen::CodeGen;

fn main() {
    let app  = App::new("rust-xdr")
        .version("0.0.1")
        .author("Ben Brittain")
        .about("Rust Generator for XDR services")
        .arg(Arg::with_name("service")
             .help("Generate service definitions in addition to types")
             .long("service")
             .short("s")
             .takes_value(false)
             .required(false))
        .arg(Arg::with_name("input")
             .help("List of XDR definition files")
             .long("input")
             .short("i")
             .multiple(true)
             .takes_value(true)
             .required(false))
        .arg(Arg::with_name("output")
             .help("Output directory")
             .long("output")
             .short("o")
             .multiple(false)
             .takes_value(true)
             .required(true))
        .get_matches();

    let files: Vec<&str> = app.values_of("input").unwrap().collect();
    let out_dir: String = app.values_of("output").unwrap().collect();

    let mut source = String::new();
    for file in files.iter() {
        let mut fin = File::open(file).expect("input file does not exist.");
        let _ = fin.read_to_string(&mut source);
    }

    let mut types_buffer = Vec::new();
    let mut proto_buffer = Vec::new();
    let mut codec_buffer = Vec::new();
    {
        let mut types_wr = CodeWriter::new(&mut types_buffer);
        let mut proto_wr = CodeWriter::new(&mut proto_buffer);
        let mut codec_wr = CodeWriter::new(&mut codec_buffer);

        let mut cg = CodeGen::new(&mut types_wr, &mut proto_wr, &mut codec_wr);
        cg.compile(source, false).expect("XDR->Rust codegen failed");
    }

    let mut types_fout = File::create(out_dir.clone() + "/types.rs")
        .expect("error creating types file");
    let _ = types_fout.write(types_buffer.as_ref());

    let mut codec_fout = File::create(out_dir.clone() + "/codec.rs")
        .expect("error creating the codec file");
    let _ = codec_fout.write(codec_buffer.as_ref());

    let mut proto_fout = File::create(out_dir.clone() + "/protocol.rs")
        .expect("error creating the protocol file");
    let _ = proto_fout.write(proto_buffer.as_ref());
}
