extern crate serde;
extern crate serde_xdr;

use gen_test;

fn main() {

    let x: gen_test::privilege_groups_t = 42;

    let vec = serde_xdr::to_bytes(&x);
    println!("{:?}", vec);

}
