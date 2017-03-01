#[macro_use]
extern crate tarpc_bench;

extern crate futures;
extern crate tokio_core;

use std::io::{self, Read, Write};
use std::str;

use futures::{Future, Stream, Poll, Async};
use tokio_core::io::{copy, Io, write_all, read_exact};
use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::Core;

fn main() {
    use std::time;
    let n = 100000;

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let addr = "127.0.0.1:12345".parse().unwrap();
    let sock = TcpListener::bind(&addr, &handle).unwrap();

    // server just copies
    let server = sock.incoming()
        .for_each(move |(sock, _)| {
            handle.spawn(MyServer {
                socket: sock,
                buf: String::new(),
                output: Vec::new(),
            });
            // //sock.set_nodelay(true).unwrap();
            // let (reader, writer) = sock.split();
            // let echo = copy(reader, writer).then(move |_| Ok(()));
            // handle.spawn(echo);
            Ok(())
        })
        .map_err(|_| ());

    core.handle().spawn(server);

    let client = TcpStream::connect(&addr, &core.handle());
    let client = core.run(client).unwrap();
    //client.set_nodelay(true).unwrap();
    let (mut r, mut w) = client.split();

    let start = time::Instant::now();
    let mut buf = [0; 2];
    for _ in 0..n {
        w = core.run(write_all(w, b"1\n")).unwrap().0;
        r = core.run(read_exact(r, &mut buf)).unwrap().0;
        assert!(buf[1] == b'\n');
        assert_eq!(str::from_utf8(&buf[..1]).unwrap().parse::<u64>().unwrap(), 2u64);
    }
    drop(r);
    drop(w);

    println!("tokio {:.0}µs/call",
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);
}

struct MyServer {
    socket: TcpStream,
    buf: String,
    output: Vec<u8>,
}

impl Future for MyServer {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        loop {
            while self.output.len() > 0 {
                match self.socket.write(&self.output) {
                    Ok(n) => { self.output.drain(..n); }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(e) => panic!("write error: {}", e),
                }
            }
            if self.buf.find("\n").is_none() {
                match self.socket.read_to_string(&mut self.buf) {
                    Ok(0) if self.buf.len() == 0 => return Ok(().into()),
                    Ok(_) => {}
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(e) => panic!("error: {}", e),
                }
            }
            let i = match self.buf.find("\n") {
                Some(i) => i,
                None => return Ok(Async::NotReady),
            };

            let integer: u64 = self.buf[..i].parse().unwrap();
            self.buf.drain(..i+1);
            writeln!(self.output, "{}", i * 2);
        }
    }
}
