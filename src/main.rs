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
        .arg(Arg::with_name("output directory")
             .help("Output directory")
             .long("output")
             .short("o")
             .takes_value(true)
             .required(true))
        .get_matches();

    let files: Vec<&str> = app.values_of("input").unwrap().collect();

    let mut source = String::new();
    for file in files.iter() {
        println!("{}", file);
        let mut fin = File::open(file).expect("input file does not exist.");
        let _ = fin.read_to_string(&mut source);
    }

    let mut buffer= Vec::new();
    {
        let mut wr = CodeWriter::new(&mut buffer);
        codegen::compile(&mut wr, source).expect("XDR->Rust codegen failed");
    }

    io::stdout().write(buffer.as_ref());
    //let mut fout = File::create(args.arg_output).expect("error creating the module.");
    //let _ = fout.write(buffer.as_ref());
}
