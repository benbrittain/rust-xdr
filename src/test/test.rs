#[macro_use]
extern crate serde_derive;

extern crate serde;
#[macro_use]
extern crate serde_xdr;

mod gen_test;
use gen_test::*;

fn main() {
    //let test = LocationCluster {
    //    locid: 2u32,
    //    last_updated: 5774,
    //    cluster_id: 10,
    //    cluster_type: ClusterType::CT_EXPERIMENT_CLUSTER_V9
    //};

    //let mut locid_list = Vec::<LocationCluster>::new();
    //for i in 0..5 {
    //    locid_list.push(LocationCluster {
    //        locid: i as u32,
    //        last_updated: i * 16 as u32,
    //        cluster_id: i * 16 * 16 as u32,
    //        cluster_type: ClusterType::CT_EXPERIMENT_CLUSTER_V9
    //    });
    //}

    //let x: Locid = 5774;

//pub enum RpcUnionExample {
//  #[serde(rename(serialize = "0"))]
//  ExStatusOk {
//    my_struct: ExampleRes,
//  },
//  #[serde(rename(serialize = "1"))]
//  ExStatusErr {
//  },
//  #[serde(rename(serialize = "2"))]
//  ExStatusInt {
//    my_union_int: u32,
//  },
//  #[serde(rename(serialize = "3"))]
//  ExStatusBool {
//    my_union_bool: bool,
//  },
//  // Default case for the XDR Union
//  UnionDefault_ {
//  },
//}
    //let test = RpcUnionExample::ExStatusOk{
    //    my_struct: ExampleRes {
    //        my_int: 34,
    //        my_bool: false
    //    },
    //};

    let test = ExampleStatus::ExStatusBool;

    let mut bytes = Vec::<u8>::new();
    serde_xdr::to_bytes(&test, &mut bytes);

    println!("{:?}", bytes);
    let obj = serde_xdr::from_bytes::<ExampleStatus>(&bytes);
    println!("{:?}", obj);

    //let mut bytes2 = Vec::<u8>::new();
    //serde_xdr::to_bytes(&locid_list, &mut bytes2);
    //println!("{:?}", bytes2);
    //let obj = serde_xdr::from_bytes::<LocationClusterVec>(&bytes2);
    //println!("{:?}", obj);
}
