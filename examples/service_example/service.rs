use std::io;
use std::ops::DerefMut;

use tokio_core::io::EasyBuf;
use tokio_service::Service;

use futures::{future, Future, BoxFuture};
use futures_cpupool::CpuPool;

use r2d2::Pool;
use r2d2_mysql::MysqlConnectionManager;

use std::default::Default;
use mysql as my;

use xdrgen::xdr_rpc;
use xdrgen::prot::*;
use serde_xdr;


pub struct ExampledbdProgService {
    pub thread_pool: CpuPool,
    pub db_pool: Pool<MysqlConnectionManager>
}

impl ExampledbdProgService {
    pub fn exampledbd_null_v1(&self) -> BoxFuture<(), io::Error> {
        future::ok(()).boxed()
    }

    pub fn get_location_cluster_v1(&self, arg: LocationCluster) -> BoxFuture<GetLocClusterRes, io::Error> {
        let db = self.db_pool.clone();
        // Tokio implementation details
        future::err(io::Error::new(io::ErrorKind::Other, "implement me!")).boxed()
    }
}
