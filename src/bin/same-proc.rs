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
use tarpc::server;
use tarpc::util::Never;
use tarpc::util::FirstSocketAddr;

use std::sync;
use tokio_core::reactor;

// no serialization
#[derive(Clone)]
struct Preserialized {
    bytes: serde::bytes::ByteBuf,
}

impl no::FutureService for Preserialized {
    type TestFut = futures::Finished<serde::bytes::ByteBuf, Never>;
    fn test(&self, _: serde::bytes::ByteBuf) -> Self::TestFut {
        futures::finished(self.bytes.clone())
    }
}


fn main() {
    use std::time;
    let n = 1000000;

    use no::FutureServiceExt;
    let mut reactor = reactor::Core::new().unwrap();
    let server = Preserialized {
        bytes: serde::bytes::ByteBuf::from(bincode::serialize(&vec![vec![DataType::Number(0), DataType::Text(sync::Arc::new("foobar".to_owned()))]], bincode::SizeLimit::Infinite).unwrap()),
    };
    let (addr, server) = server.listen("127.0.0.1:2223".first_socket_addr(),
                &reactor.handle(),
                server::Options::default())
        .unwrap();
    reactor.handle().spawn(server);

    use client::future::ClientExt;
    let client = no::FutureClient::connect(addr,
                                           client::Options::default().handle(reactor.handle()));
    let client = reactor.run(client).unwrap();


    let arg = serde::bytes::ByteBuf::from(bincode::serialize(&DataType::Number(0),
                                                             bincode::SizeLimit::Infinite)
        .unwrap());
    let start = time::Instant::now();
    for _ in 0..n {
        reactor.run(client.test(arg.clone())).unwrap();
    }
    println!("same {:.0}µs/call",
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);
}
