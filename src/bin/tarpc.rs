#![feature(plugin)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate tarpc_bench;

#[macro_use]
extern crate tarpc;

extern crate futures;
extern crate tokio_core;

use tarpc::future::{client, server};
use tarpc::util::Never;
use tarpc::util::FirstSocketAddr;
use tokio_core::reactor;

mod srv {
    service! {
        rpc test(i: u64) -> u64;
    }
}

#[derive(Clone)]
struct Srv;
impl srv::FutureService for Srv {
    type TestFut = futures::Finished<u64, Never>;
    fn test(&self, i: u64) -> Self::TestFut {
        futures::finished(i * 2)
    }
}


fn main() {
    use std::time;
    let n = 100000;

    use srv::FutureServiceExt;
    let mut reactor = reactor::Core::new().unwrap();
    let (handle, server) = Srv.listen("127.0.0.1:2223".first_socket_addr(),
                &reactor.handle(),
                server::Options::default())
        .unwrap();
    reactor.handle().spawn(server);

    use client::ClientExt;
    let client = srv::FutureClient::connect(handle.addr(),
                                            client::Options::default().handle(reactor.handle()));
    let client = reactor.run(client).unwrap();


    let start = time::Instant::now();
    for i in 0..n {
        reactor.run(client.test(i)).unwrap();
    }
    println!("tarpc {:.0}µs/call",
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);
}
