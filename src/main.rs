//#![feature(rustc_private)]

#[macro_use]
extern crate nom;
extern crate rustc_serialize;
extern crate docopt;
extern crate serde;
extern crate serde_xdr;

use std::fs::File;
use std::str;
use std::io::{Write, Read};
use std::io::BufWriter;

use std::path::Path;

use docopt::Docopt;

mod test;
mod gen_test;
//mod encoder;
mod parser;
mod codegen;
mod code_writer;

use code_writer::CodeWriter;

const USAGE: &'static str = "
xdrust: an XDR compiler (RFC4506 compliant + some extras) for Rust

Usage:
  xdrust <input> <output>

Options:
  -h --help     Show this screen.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_input: String,
    arg_output: String
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    let mut fin = File::open(args.arg_input).expect("input file does not exist.");
    let mut source = String::new();
    let _ = fin.read_to_string(&mut source);
    let mut buffer= Vec::new();
    {
        let mut wr = CodeWriter::new(&mut buffer);
        codegen::compile(&mut wr, source).expect("XDR->Rust codegen failed");
    }
    let mut fout = File::create(args.arg_output).expect("error creating the module.");
    fout.write(buffer.as_ref());
}
