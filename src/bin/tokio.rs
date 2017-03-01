#[macro_use]
extern crate tarpc_bench;

extern crate futures;
extern crate tokio_core;

use futures::{Future, Stream};
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
            //sock.set_nodelay(true).unwrap();
            let (reader, writer) = sock.split();
            let echo = copy(reader, writer).then(move |_| Ok(()));
            handle.spawn(echo);
            Ok(())
        })
        .map_err(|_| ());

    core.handle().spawn(server);

    let client = TcpStream::connect(&addr, &core.handle());
    let client = core.run(client).unwrap();
    //client.set_nodelay(true).unwrap();
    let (r, w) = client.split();
    let mut r = Some(r);
    let mut w = Some(w);

    let start = time::Instant::now();
    let mut buf = Vec::from("foobar".as_bytes());
    for _ in 0..n {
        let w_ = w.take().unwrap();
        w = Some(core.run(write_all(w_, &buf)).unwrap().0);
        let r_ = r.take().unwrap();
        r = Some(core.run(read_exact(r_, &mut buf)).unwrap().0);
    }
    drop(r);
    drop(w);

    println!("tokio {:.0}µs/call",
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);
}
