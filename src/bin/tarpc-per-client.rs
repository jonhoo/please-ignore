#![feature(plugin)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate tarpc_bench;

extern crate bincode;
extern crate futures;
#[macro_use]
extern crate tarpc;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

use futures::Stream;
use tokio_service::NewService;


use std::thread;
use tarpc::future::{client, server};
use tarpc::util::Never;
use tarpc::util::FirstSocketAddr;
use tokio_core::reactor;
use std::io;
use tokio_proto::BindServer;

mod srv {
    service! {
        rpc test(i: u64) -> u64;
    }
}

#[derive(Clone)]
struct Srv;
impl srv::FutureService for Srv {
    type TestFut = Result<u64, Never>;
    fn test(&self, i: u64) -> Self::TestFut {
        Ok(i * 2)
    }
}

fn main() {
    use std::time;
    let n = 100000;
    let c = 3;
    let addr = "127.0.0.1:7007".first_socket_addr();

    // start server thread
    thread::spawn(move || {
        let mut reactor = reactor::Core::new().unwrap();
        let listener = server::listener(&addr, &reactor.handle()).unwrap();
        let server = listener.incoming().for_each(move |(socket, _)| {
            thread::spawn(move || -> Result<(), io::Error> {
                use srv::FutureServiceExt;
                let server = Srv.to_new_service();
                let mut reactor = reactor::Core::new().unwrap();
                tarpc::protocol::Proto::new()
                    .bind_server(&reactor.handle(), socket, server.new_service()?);
                loop {
                    reactor.turn(None)
                }
            });
            Ok(())
        });
        reactor.run(server).unwrap();
    });

    thread::sleep(time::Duration::from_secs(1));

    let start = time::Instant::now();
    let clients: Vec<_> = (0..c)
        .into_iter()
        .map(|_| {
            thread::spawn(move || {
                use client::ClientExt;
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

    println!("tarpc-per-client-{} {:.0}µs/call",
             c,
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);
}
