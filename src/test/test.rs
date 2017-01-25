#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_xdr;

mod gen_test;
use gen_test::*;

fn main() {
    let test = LocationCluster {
        locid: 2u32,
        last_updated: 5774,
        cluster_id: 10,
        cluster_type: ClusterType::CT_EXPERIMENT_CLUSTER_V9
    };

    let mut locid_list = Vec::<LocationCluster>::new();
    for i in 0..5 {
        locid_list.push(LocationCluster {
            locid: i as u32,
            last_updated: i * 16 as u32,
            cluster_id: i * 16 * 16 as u32,
            cluster_type: ClusterType::CT_EXPERIMENT_CLUSTER_V9
        });
    }

    let x: Locid = 5774;


    let bytes = serde_xdr::to_bytes(&test).unwrap();
    println!("{:?}", bytes);
    let obj = serde_xdr::from_bytes::<LocationCluster>(&bytes);
    println!("{:?}", obj);

    let bytes = serde_xdr::to_bytes(&locid_list).unwrap();
    println!("{:?}", bytes);
    let obj = serde_xdr::from_bytes::<LocationClusterVec>(&bytes);
    println!("{:?}", obj);
}
