#[macro_use]
extern crate tarpc_bench;
extern crate memcached;

use memcached::Client;
use memcached::proto::{Operation, ProtoType};

fn main() {
    use std::time;
    use std::thread;
    let n = 100000;
    let c = 3;

    let servers = [("tcp://127.0.0.1:2222", 1)];
    let mut client = Client::connect(&servers, ProtoType::Binary).unwrap();
    client.set(b"1", b"[1,foobar]", 0xdeadbeef, 2).unwrap();
    drop(client);

    let start = time::Instant::now();
    let clients: Vec<_> = (0..c)
        .into_iter()
        .map(|_| {
            thread::spawn(move || {
                let mut client = Client::connect(&servers, ProtoType::Binary).unwrap();
                for _ in 0..n {
                    match client.get(b"1") {
                        _ => (),
                    }
                }
            })
        })
        .collect();

    for c in clients {
        c.join().unwrap();
    }

    println!("memcached-multi-{} {:.0}µs/call",
             c,
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);
}
