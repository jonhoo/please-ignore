#![feature(plugin)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate tarpc;

extern crate futures;
extern crate tokio_core;
extern crate serde;

extern crate bincode;

use std::sync;


#[macro_export]
macro_rules! dur_to_ns {
    ($d:expr) => {{
        const NANOS_PER_SEC: u64 = 1_000_000_000;
        let d = $d;
        d.as_secs() * NANOS_PER_SEC + d.subsec_nanos() as u64
    }}
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DataType {
    None,
    Number(i64),
    Text(sync::Arc<String>),
}

pub mod no {
    use super::serde;
    service! {
        rpc test(i: serde::bytes::ByteBuf) -> serde::bytes::ByteBuf;
    }
}
