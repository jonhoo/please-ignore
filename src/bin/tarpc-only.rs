#![feature(plugin)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate tarpc_bench;

extern crate bincode;
extern crate futures;
#[macro_use]
extern crate tarpc;
extern crate tokio_core;

use std::net::TcpListener;
use std::net::TcpStream;


use futures::Future;
use std::thread;
use tarpc::future::{client, server};
use tarpc::util::Never;
use tarpc::util::FirstSocketAddr;
use tokio_core::reactor;
use std::io;
use std::io::Read;
use std::io::Write;
use tarpc::serde::Deserialize;
use tarpc::serde::Serialize;
use tokio_core::io::Codec;
use tokio_core::io::EasyBuf;

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


fn inner<S, Req, Resp, E>(s: &S, mut stream: TcpStream)
    where S: tarpc::tokio_service::Service<Request = Result<Req, bincode::Error>,
                                           Response = server::Response<Resp, E>,
                                           Error = io::Error> + 'static,
          Req: Deserialize + 'static,
          Resp: Serialize + 'static,
          E: Serialize + 'static,
          S: tarpc::tokio_service::Service
{
    let mut codec: tarpc::protocol::Codec<S::Response, Req> = tarpc::protocol::Codec::new();
    let mut v = Vec::with_capacity(4096);
    let mut ebuf = EasyBuf::new();
    let mut out = Vec::new();

    loop {
        // read some bytes
        v.resize(4096, 0);
        match stream.read(v.as_mut_slice()) {
            Ok(rn) if rn == 0 => break,
            Ok(rn) => {
                v.resize(rn, 0);
            }
            Err(_) => break,
        }
        ebuf.get_mut().append(&mut v);

        // try to decode what we have so far
        match codec.decode(&mut ebuf) {
            Ok(Some((id, req))) => {
                let res = s.call(req).wait().unwrap();

                codec.encode((id, res), &mut out).unwrap();
                stream.write(&out[..]).unwrap();
                out.clear();
            }
            Ok(None) => print!("ok none\n"),
            Err(e) => print!("err {:?}\n", e),
        }
    }
}

fn main() {
    use std::time;
    let n = 10000;
    let addr = "127.0.0.1:7007".first_socket_addr();

    // start server thread
    let listener = TcpListener::bind(addr).unwrap();
    thread::spawn(move || for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // server spawns a thread for each connection
                thread::spawn(move || { inner(&srv::tarpc_service_AsyncServer__(Srv), stream); });
            }
            Err(e) => {
                print!("accept failed {:?}\n", e);
            }
        }
    });

    // make a client
    use client::ClientExt;
    let mut reactor = reactor::Core::new().unwrap();
    let client = srv::FutureClient::connect(addr,
                                            client::Options::default().handle(reactor.handle()));
    let client = reactor.run(client).unwrap();

    let start = time::Instant::now();
    for i in 0..n {
        reactor.run(client.test(i)).unwrap();
    }
    println!("tarpc-only {:.0}µs/call",
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);
}
