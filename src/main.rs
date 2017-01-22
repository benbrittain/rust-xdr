#![feature(rustc_private)]

#[macro_use]
extern crate nom;

use std::fs::File;
use std::str;
use std::io::{Write, Read};
use std::io::BufWriter;

mod parser;
mod codegen;
mod code_writer;
use code_writer::CodeWriter;

use std::path::Path;

fn main() {
    let path = Path::new("shared_defs.x");
    let file = path.file_name().unwrap();
    println!("{:?}", file);

    let mut vec = Vec::new();

    {
        let mut wr = CodeWriter::new(&mut vec);
        codegen::compile(&mut wr, path).expect("XDR->Rust codegen failed");
    }

    println!("{:?}", str::from_utf8(vec.as_ref()).unwrap().to_string());
}
