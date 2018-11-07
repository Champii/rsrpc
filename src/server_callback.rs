use std::net::SocketAddr;
use std::sync::Arc;

use super::proto::Packet;

#[derive(Clone)]
pub struct ServerCallback {
  pub closure: Arc<Fn(Packet, SocketAddr) -> Packet>,
}

impl ServerCallback {
  pub fn new(closure: Arc<Fn(Packet, SocketAddr) -> Packet>) -> ServerCallback {
    ServerCallback { closure }
  }

  pub fn new_empty() -> ServerCallback {
    Self::new(Arc::new(|a, _| a))
  }
}

unsafe impl Send for ServerCallback {}
unsafe impl Sync for ServerCallback {}
