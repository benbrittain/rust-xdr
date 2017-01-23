#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_xdr;

mod gen_test;

fn main() {
//    let x: gen_test::PrivilegeGroupsT = 42;
    let vec = serde_xdr::to_bytes(&gen_test::ModeratorPrivilegeTypeT::MODPRIVFLAGMOD);
    println!("{:?}", vec);
}
