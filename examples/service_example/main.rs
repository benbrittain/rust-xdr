#![feature(custom_attribute, custom_derive, plugin)]
#[macro_use]
extern crate mysql;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_xdr;
extern crate futures;
extern crate futures_cpupool;
extern crate r2d2;
extern crate r2d2_mysql;
extern crate serde;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

use futures_cpupool::CpuPool;
use r2d2::Pool;
use r2d2_mysql::{CreateManager, MysqlConnectionManager};
use tokio_proto::TcpServer;

mod xdrgen;
use xdrgen::codec::ExampledbdProtocol;

mod service;
use service::ExampledbdProgService;

const DB_URL: &'static str = "mysql://some_db";

fn main() {
    let addr = "0.0.0.0:28755".parse().unwrap(); // that's the port for the service
    let thread_pool = CpuPool::new(2);

    let db_config = r2d2::Config::default();
    let db_manager = MysqlConnectionManager::new(DB_URL).unwrap();
    let db_pool = r2d2::Pool::new(db_config, db_manager).unwrap();

    TcpServer::new(ExampledbdProtocol, addr).serve(move || {
        Ok (ExampledbdProgService {
            thread_pool: thread_pool.clone(),
            db_pool: db_pool.clone()
        })
    })
}
