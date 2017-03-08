#![feature(plugin)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate tarpc_bench;

extern crate bincode;
extern crate futures;
#[macro_use]
extern crate tarpc;
extern crate tokio_core;

use tokio_core::net::TcpListener;
use tokio_core::net::TcpStream;


use futures::Future;
use std::thread;
use tarpc::future::{client, server};
use tarpc::util::Never;
use tarpc::util::FirstSocketAddr;
use tokio_core::reactor;
use std::io;
use tarpc::serde::Deserialize;
use tarpc::serde::Serialize;
use tokio_core::io::Io;
use futures::Stream;

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


fn inner<S, Req, Resp, E>(s: S, stream: TcpStream)
    where S: tarpc::tokio_service::Service<Request = Result<Req, bincode::Error>,
                                           Response = server::Response<Resp, E>,
                                           Error = io::Error> + 'static,
          Req: Deserialize + 'static,
          Resp: Serialize + 'static,
          E: Serialize + 'static,
          S: tarpc::tokio_service::Service
{
    let (mut tcp_out, tcp_in) = stream.framed(tarpc::protocol::Codec::new()).split();
    let mut reactor = reactor::Core::new().unwrap();
    let (tx, mut rx) = futures::unsync::mpsc::unbounded();
    let requests = tcp_in.for_each(move |(id, req)| {
        let tx = tx.clone();
        s.call(req).then(move |resp| Ok(tx.send((id, resp)).unwrap()))
    });
    reactor.handle().spawn(requests.map_err(|_| ()));
    loop {
        match reactor.run(rx.into_future()) {
            Ok((Some((id, res)), rx2)) => {
                use futures::Sink;
                rx = rx2;
                tcp_out = tcp_out.send((id, res.unwrap())).wait().unwrap();
            }
            Ok((None, rx2)) => {
                rx = rx2;
            }
            Err((_, rx2)) => {
                rx = rx2;
            }
        }
    }
}

fn main() {
    use std::time;
    let n = 100000;
    let c = 3;
    let addr = "127.0.0.1:7009".first_socket_addr();

    // start server thread
    thread::spawn(move || {
        let mut reactor = reactor::Core::new().unwrap();
        let listener = TcpListener::bind(&addr, &reactor.handle()).unwrap();
        let server = listener.incoming().for_each(|(stream, _)| {
            // server spawns a thread for each connection
            thread::spawn(move || { inner(srv::tarpc_service_AsyncServer__(Srv), stream); });
            Ok(())
        });
        reactor.run(server).unwrap();
    });

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

    println!("tarpc-fut-multi-{} {:.0}µs/call",
             c,
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);
}
