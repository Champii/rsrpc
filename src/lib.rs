#![feature(async_await, await_macro, pin, arbitrary_self_types, futures_api)]
#![recursion_limit="128"]
#[macro_use] extern crate serde_derive;
#[macro_use] pub extern crate lazy_static;
#[macro_use] extern crate log;

pub extern crate bincode;
pub extern crate byteorder;
extern crate env_logger;
extern crate serde;
extern crate serde_bytes;
extern crate futures;
extern crate sha2;
extern crate hex;
extern crate tokio_core;
extern crate pin_utils;

#[macro_use] pub mod service_macro;
mod async_response_matcher;
pub mod network;
pub mod transport;
pub mod utils;
pub mod proto;
pub mod tests;
pub mod server_callback;
pub mod plugins;

pub use std::collections::HashMap;
pub use std::sync::{ Mutex };
pub use futures::channel::oneshot;
pub use futures::executor::block_on;
pub use bincode::{ serialize, deserialize };
pub use std::net::SocketAddr;
pub use std::thread;

pub use self::proto::{Packet};
pub use self::utils::*;
pub use self::network::{Network};
pub use self::server_callback::{ServerCallback};
pub use self::transport::*;
pub use self::plugins::*;
pub use self::async_response_matcher::{AsyncResponseMatcher};
pub use self::service_macro::*;
