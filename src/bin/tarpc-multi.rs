#![feature(plugin)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate tarpc_bench;

#[macro_use]
extern crate tarpc;

extern crate futures;
extern crate tokio_core;

use tarpc::client;
use tarpc::server;
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
    use std::thread;
    use std::time;
    let n = 100000;
    let s = 1;
    let c = 3;

    let addr = "127.0.0.1:2223".first_socket_addr();
    for _ in 0..s {
        thread::spawn(move || {
            use srv::FutureServiceExt;
            let mut reactor = reactor::Core::new().unwrap();
            let (_, server) = Srv.listen(addr, &reactor.handle(), server::Options::default())
                .unwrap();
            reactor.handle().spawn(server);
            loop {
                reactor.turn(None);
            }
        });
    }

    let start = time::Instant::now();
    let clients: Vec<_> = (0..c)
        .into_iter()
        .map(|_| {
            thread::spawn(move || {
                use client::future::ClientExt;
                let mut reactor = reactor::Core::new().unwrap();
                let client = srv::FutureClient::connect(addr,
                                                        client::Options::default()
                                                            .handle(reactor.handle()));
                let client = reactor.run(client).unwrap();

                for i in 0..n {
                    reactor.run(client.test(i)).unwrap();
                }
            })
        })
        .collect();

    for c in clients {
        c.join().unwrap();
    }

    println!("tarpc-multi-{} {:.0}µs/call",
             c,
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);
}
