#[macro_use]
extern crate tarpc_bench;
extern crate memcached;

use memcached::Client;
use memcached::proto::{Operation, ProtoType};

fn main() {
    use std::time;
    let n = 100000;

    // memcached
    let servers = [("tcp://127.0.0.1:2222", 1)];
    let mut client = Client::connect(&servers, ProtoType::Binary).unwrap();
    client.set(b"1", b"[1,foobar]", 0xdeadbeef, 2).unwrap();

    let start = time::Instant::now();
    for _ in 0..n {
        match client.get(b"1") {
            _ => (),
        }
    }
    println!("memcached {:.0}µs/call",
             dur_to_ns!(start.elapsed()) as f64 / n as f64 / 1000.0);
}
