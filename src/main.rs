#![feature(plugin)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate tarpc;

extern crate futures;
extern crate tokio_core;
extern crate serde;

extern crate bincode;

use std::sync::mpsc;
use std::sync;
use std::thread;
use tarpc::{client, server};
use tarpc::client::sync::ClientExt;
use tarpc::util::Never;

#[derive(Debug, Serialize, Deserialize)]
pub enum DataType {
    None,
    Number(i64),
    Text(sync::Arc<String>),
}

#[derive(Clone)]
struct HelloServer;

impl HelloServer {
    pub fn inner(&self, i: DataType) -> Result<Vec<Vec<DataType>>, Never> {
        // some random stuff
        Ok(vec![vec![i, DataType::Text(sync::Arc::new("foobar".to_owned()))]])
    }
}

// the naive version
mod naive {
    use super::DataType;
    service! {
        rpc test(i: DataType) -> Vec<Vec<DataType>>;
    }
}

impl naive::SyncService for HelloServer {
    fn test(&self, i: DataType) -> Result<Vec<Vec<DataType>>, Never> {
        self.inner(i)
    }
}

// manual serialization
mod manual {
    use super::DataType;
    use super::serde;
    service! {
        rpc test(i: serde::bytes::ByteBuf) -> serde::bytes::ByteBuf;
    }
}

impl manual::SyncService for HelloServer {
    fn test(&self, i: serde::bytes::ByteBuf) -> Result<serde::bytes::ByteBuf, Never> {
        self.inner(bincode::deserialize(&i[..]).unwrap())
            .map(|r| {
                serde::bytes::ByteBuf::from(bincode::serialize(&r, bincode::SizeLimit::Infinite)
                    .unwrap())
            })
    }
}

// no serialization
#[derive(Clone)]
struct Preserialized {
    bytes: serde::bytes::ByteBuf,
}
mod no {
    use super::DataType;
    use super::serde;
    service! {
        rpc test(i: serde::bytes::ByteBuf) -> serde::bytes::ByteBuf;
    }
}

impl no::SyncService for Preserialized {
    fn test(&self, i: serde::bytes::ByteBuf) -> Result<serde::bytes::ByteBuf, Never> {
        Ok(self.bytes.clone())
    }
}

const NANOS_PER_SEC: u64 = 1_000_000_000;
macro_rules! dur_to_ns {
    ($d:expr) => {{
        let d = $d;
        d.as_secs() * NANOS_PER_SEC + d.subsec_nanos() as u64
    }}
}

fn main() {
    use std::time;
    let n = 100000;

    // local operation
    let s = HelloServer;
    let start = time::Instant::now();
    for i in 0..n {
        s.inner(DataType::Number(i)).unwrap();
    }
    println!("local {:.0}ns/call",
             dur_to_ns!(start.elapsed()) as f64 / n as f64);

    // local operation with serialization
    let s = HelloServer;
    let start = time::Instant::now();
    for i in 0..n {
        let arg = serde::bytes::ByteBuf::from(bincode::serialize(&DataType::Number(i),
                                                                 bincode::SizeLimit::Infinite)
            .unwrap());
        let res = s.inner(bincode::deserialize(&arg[..]).unwrap()).unwrap();
        let res = serde::bytes::ByteBuf::from(bincode::serialize(&res,
                                                                 bincode::SizeLimit::Infinite)
            .unwrap());
        let res: Vec<Vec<DataType>> = bincode::deserialize(&res[..]).unwrap();
    }
    println!("local-serialize {:.0}ns/call",
             dur_to_ns!(start.elapsed()) as f64 / n as f64);

    // loopback server with naive types
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        use naive::SyncServiceExt;
        let mut handle = HelloServer.listen("localhost:0", server::Options::default()).unwrap();
        tx.send(handle.addr()).unwrap();
        handle.run();
    });
    let mut client = naive::SyncClient::connect(rx.recv().unwrap(), client::Options::default())
        .unwrap();

    let start = time::Instant::now();
    for i in 0..n {
        client.test(DataType::Number(i)).unwrap();
    }
    println!("loopback-naive {:.0}µs/call",
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);

    // loopback server with manual serialization
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        use manual::SyncServiceExt;
        let mut handle = HelloServer.listen("localhost:0", server::Options::default()).unwrap();
        tx.send(handle.addr()).unwrap();
        handle.run();
    });
    let mut client = manual::SyncClient::connect(rx.recv().unwrap(), client::Options::default())
        .unwrap();

    let start = time::Instant::now();
    for i in 0..n {
        let arg = serde::bytes::ByteBuf::from(bincode::serialize(&DataType::Number(i),
                                                                 bincode::SizeLimit::Infinite)
            .unwrap());
        let res = client.test(arg).unwrap();
        let res: Vec<Vec<DataType>> = bincode::deserialize(&res[..]).unwrap();
    }
    println!("loopback-manual {:.0}µs/call",
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);

    // loopback server with no serialization
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        use no::SyncServiceExt;
        let mut handle = Preserialized {
            bytes: serde::bytes::ByteBuf::from(bincode::serialize(&vec![vec![DataType::Number(0), DataType::Text(sync::Arc::new("foobar".to_owned()))]], bincode::SizeLimit::Infinite).unwrap()),
        }.listen("localhost:0", server::Options::default()).unwrap();
        tx.send(handle.addr()).unwrap();
        handle.run();
    });
    let mut client = manual::SyncClient::connect(rx.recv().unwrap(), client::Options::default())
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
