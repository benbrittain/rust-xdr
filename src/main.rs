//#![feature(rustc_private)]

#[macro_use]
extern crate nom;
extern crate rustc_serialize;
extern crate serde;
#[macro_use]
extern crate serde_xdr;

use std::env;
use std::fs::File;
use std::str;
use std::io::{self, Write, Read};

mod parser;
mod codegen;
mod code_writer;
mod function_writer;

use code_writer::CodeWriter;

const USAGE: &'static str = "
xdrust: an XDR compiler (RFC4506 compliant + some extras) for Rust

Usage:
  xdrust <input> <output>

Options:
  -h --help     Show this screen.
";

fn main() {
    let args: Vec<_> = env::args().collect();

    let mut source = String::new();
    for file in args {
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
