#![feature(plugin)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate tarpc_bench;
use tarpc_bench::no;

extern crate tarpc;

extern crate futures;
extern crate tokio_core;
extern crate serde;

extern crate bincode;

use tarpc::client;
use tarpc::server;
use tarpc::util::Never;
use tarpc::util::FirstSocketAddr;

use tokio_core::reactor;

// no serialization
#[derive(Clone, Copy)]
struct Doubler;

impl no::FutureService for Doubler {
    type DoubleFut = Result<i32, Never>;
    fn double(&self, i: i32) -> Self::DoubleFut {
        Ok(i * 2)
    }
}


fn main() {
    use std::time;
    let n = 1000000;

    use no::FutureServiceExt;
    let mut reactor = reactor::Core::new().unwrap();
    let server = Doubler;
    let (addr, server) = server.listen("127.0.0.1:2223".first_socket_addr(),
                &reactor.handle(),
                server::Options::default())
        .unwrap();
    reactor.handle().spawn(server);

    use client::future::ClientExt;
    let client = no::FutureClient::connect(addr,
                                           client::Options::default().handle(reactor.handle()));
    let client = reactor.run(client).unwrap();


    let start = time::Instant::now();
    for i in 0..n {
        reactor.run(client.double(i)).unwrap();
    }
    println!("same {:.0}µs/call",
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);
}
