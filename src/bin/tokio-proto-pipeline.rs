#[macro_use]
extern crate tarpc_bench;

extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate net2;

use std::str;
use std::io::{self, ErrorKind, Write};

use futures::{future, Future, BoxFuture};
use tokio_core::io::{Io, Codec, Framed, EasyBuf};
use tokio_core::net::TcpListener;
use tokio_proto::pipeline::{ServerProto, ClientProto};
use tokio_proto::TcpClient;
use tokio_service::Service;
use tokio_core::reactor::Core;
use futures::stream::Stream;
use tokio_proto::BindServer;
use std::net::SocketAddr;

// First, we implement a *codec*, which provides a way of encoding and
// decoding messages for the protocol. See the documentation for `Codec` in
// `tokio-core` for more details on how that works.

#[derive(Default)]
pub struct IntCodec;

fn parse_u64(from: &[u8]) -> Result<u64, io::Error> {
    Ok(str::from_utf8(from).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?
        .parse()
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?)
}

impl Codec for IntCodec {
    type In = u64;
    type Out = u64;

    // Attempt to decode a message from the given buffer if a complete
    // message is available; returns `Ok(None)` if the buffer does not yet
    // hold a complete message.
    fn decode(&mut self, buf: &mut EasyBuf) -> Result<Option<u64>, io::Error> {
        if let Some(i) = buf.as_slice().iter().position(|&b| b == b'\n') {
            // remove the line, including the '\n', from the buffer
            let full_line = buf.drain_to(i + 1);

            // strip the'`\n'
            let slice = &full_line.as_slice()[..i];

            Ok(Some(parse_u64(slice)?))
        } else {
            Ok(None)
        }
    }

    // Attempt to decode a message assuming that the given buffer contains
    // *all* remaining input data.
    fn decode_eof(&mut self, buf: &mut EasyBuf) -> io::Result<u64> {
        let amt = buf.len();
        Ok(parse_u64(buf.drain_to(amt).as_slice())?)
    }

    fn encode(&mut self, item: u64, into: &mut Vec<u8>) -> io::Result<()> {
        writeln!(into, "{}", item).map(|_| ())
    }
}

// Next, we implement the server protocol, which just hooks up the codec above.

pub struct IntProto;

impl<T: Io + 'static> ServerProto<T> for IntProto {
    type Request = u64;
    type Response = u64;
    type Transport = Framed<T, IntCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(IntCodec))
    }
}

impl<T: Io + 'static> ClientProto<T> for IntProto {
    type Request = u64;
    type Response = u64;
    type Transport = Framed<T, IntCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(IntCodec))
    }
}

// Now we implement a service we'd like to run on top of this protocol

pub struct Doubler;

impl Service for Doubler {
    type Request = u64;
    type Response = u64;
    type Error = io::Error;
    type Future = future::FutureResult<u64, io::Error>;

    fn call(&self, req: u64) -> Self::Future {
        // Just return the request, doubled
        future::ok(req * 2)
    }
}

// Finally, we can actually host this service locally!
fn main() {
    use std::time;
    let n = 100000;

    let addr = "127.0.0.1:12345".parse().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // start server
    // thread::spawn(move || TcpServer::new(IntProto, addr).serve(|| Ok(Doubler)));
    let listener = match addr {
        SocketAddr::V4(_) => net2::TcpBuilder::new_v4().unwrap(),
        SocketAddr::V6(_) => net2::TcpBuilder::new_v6().unwrap(),
    };
    listener.reuse_address(true).unwrap();
    listener.bind(addr).unwrap();
    let server = listener.listen(1024)
        .and_then(|l| TcpListener::from_listener(l, &addr, &handle))
        .unwrap()
        .incoming()
        .for_each(move |(socket, _)| {
            IntProto.bind_server(&handle, socket, Doubler);
            Ok(())
        })
        .map_err(|_| ());

    core.handle().spawn(server);

    // connect with client
    let handle = core.handle();
    let client = core.run(TcpClient::new(IntProto).connect(&addr, &handle)).unwrap();

    // benchmark
    let start = time::Instant::now();
    for _ in 0..n {
        core.run(client.call(1)).unwrap();
    }

    println!("tokio-proto-pipeline {:.0}µs/call",
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);
}
