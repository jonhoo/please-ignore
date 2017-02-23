#![feature(plugin)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate tarpc_bench;
use tarpc_bench::{no, DataType};

extern crate tarpc;

extern crate futures;
extern crate tokio_core;
extern crate serde;

extern crate bincode;

use tarpc::client;
use tarpc::client::sync::ClientExt;
use tarpc::util::FirstSocketAddr;

fn main() {
    use std::time;
    let n = 100000;

    let mut client = no::SyncClient::connect("127.0.0.1:2223".first_socket_addr(),
                                             client::Options::default())
        .unwrap();

    let arg = serde::bytes::ByteBuf::from(bincode::serialize(&DataType::Number(0),
                                                             bincode::SizeLimit::Infinite)
        .unwrap());
    let start = time::Instant::now();
    for _ in 0..n {
        client.test(arg.clone()).unwrap();
    }
    println!("loopback-no {:.0}µs/call",
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);
}
