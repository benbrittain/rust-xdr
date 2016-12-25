#![cfg_attr(not(feature = "with-syntex"), feature(rustc_private))]
#[macro_use]
extern crate nom;
extern crate aster;

#[cfg(feature = "with-syntex")]
extern crate syntex_syntax as syntax;

#[cfg(not(feature = "with-syntex"))]
extern crate syntax;

mod xdrgen;

use std::path::Path;

fn main() {
    let path = Path::new("web_prot.x");
    let file = path.file_name().unwrap();
    println!("{:?}", file);
    xdrgen::compile(path).expect("XDR->Rust codegen failed");
}
