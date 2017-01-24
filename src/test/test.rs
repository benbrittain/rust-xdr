#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_xdr;

mod gen_test;
use gen_test::*;

fn main() {
    let test = LocationCluster {
        locid: 2i8,
        last_updated: 5774,
        cluster_id: 10,
        cluster_type: ClusterType::CtExperimentClusterV9
    };

    let x: Locid = 5774;

    let bytes = serde_xdr::to_bytes(&test).unwrap();
    println!("{:?}", bytes);
    let obj = serde_xdr::from_bytes::<LocationCluster>(&bytes);
    println!("{:?}", obj);
}
