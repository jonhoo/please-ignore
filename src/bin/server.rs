#![feature(plugin)]
#![plugin(tarpc_plugins)]

extern crate tarpc_bench;
use tarpc_bench::{no, DataType};

extern crate tarpc;

extern crate futures;
extern crate tokio_core;
extern crate serde;

extern crate bincode;

use std::sync;
use tarpc::server;
use tarpc::util::Never;

// no serialization
#[derive(Clone)]
struct Preserialized {
    bytes: serde::bytes::ByteBuf,
}

impl no::SyncService for Preserialized {
    fn test(&self, _: serde::bytes::ByteBuf) -> Result<serde::bytes::ByteBuf, Never> {
        Ok(self.bytes.clone())
    }
}

fn main() {
    use no::SyncServiceExt;
    let mut handle = Preserialized {
        bytes: serde::bytes::ByteBuf::from(bincode::serialize(&vec![vec![DataType::Number(0), DataType::Text(sync::Arc::new("foobar".to_owned()))]], bincode::SizeLimit::Infinite).unwrap()),
    }.listen("127.0.0.1:2223", server::Options::default()).unwrap();
    handle.run();
}
