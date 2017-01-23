#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_xdr;

mod gen_test;
use gen_test::*;

fn main() {
//    let x: gen_test::PrivilegeGroupsT = 42;
    let test = LocationCluster {
        locid: 23u32,
        last_updated: 0,
        cluster_id: 10,
        cluster_type: ClusterType::CT_EXPERIMENT_CLUSTER_V1
    };

    let bytes = serde_xdr::to_bytes(&test);
 //   let obj, bytes_consumed = serde_xdr::from_bytes(&bytes);
//
    println!("{:?}", bytes);
}
