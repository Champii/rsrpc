#![feature(
    async_await,
    await_macro,
    pin,
    arbitrary_self_types,
    futures_api,
    duration_as_u128,
    deadline_api
)]
#![recursion_limit = "128"]
#[macro_use]
extern crate serde_derive;
#[macro_use]
pub extern crate lazy_static;
#[macro_use]
extern crate log;

pub extern crate bincode;
pub extern crate byteorder;
extern crate futures;
extern crate hex;
extern crate pin_utils;
extern crate serde;
extern crate serde_bytes;
extern crate sha2;
extern crate tokio_core;

#[macro_use]
pub mod service_macro;
mod async_response_matcher;
pub mod network;
pub mod plugins;
pub mod proto;
pub mod server_callback;
pub mod tests;
pub mod timer;
pub mod transport;
pub mod utils;

pub use bincode::{deserialize, serialize};
pub use futures::channel::oneshot;
pub use futures::executor::block_on;
pub use std::collections::HashMap;
pub use std::net::SocketAddr;
pub use std::sync::Mutex;
pub use std::thread;

pub use self::async_response_matcher::AsyncResponseMatcher;
pub use self::network::Network;
pub use self::plugins::*;
pub use self::proto::Packet;
pub use self::server_callback::ServerCallback;
pub use self::service_macro::*;
pub use self::transport::*;
pub use self::utils::*;
