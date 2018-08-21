use std::thread;
use std::net::{ SocketAddr };
use bincode::{ serialize };
use std::sync::{ Arc };

use super::proto::Packet;
use super::transport::*;

pub struct ServerCallback {
  pub closure: Box<Fn(Vec<u8>) -> Vec<u8>>,
}

unsafe impl Send for ServerCallback {}
unsafe impl Sync for ServerCallback {}

#[derive(Clone)]
pub struct Network<T: Transport + Clone> {
  pub running: bool,
  pub transport: Box<T>,
  pub callback: Arc<ServerCallback>,
}

impl<T: Transport + Clone> Clone for Network<T> {
  fn clone(&self) -> Self {
    Network {
      running: self.running.clone(),
      transport: self.transport.clone(),
      callback: self.callback.clone(),
    }
  }
}

impl<T: Transport + Clone> Network<T> {
  pub fn new(transport: T, callback: ServerCallback) -> Network<T> {
    Network {
      transport: Box::new(transport),
      running: true,
      callback: Arc::new(callback),
    }
  }


  pub fn set_callback(&mut self, callback: ServerCallback) {
    self.callback = Arc::new(callback);
  }

  pub fn send(&mut self, addr: &SocketAddr, buff: Vec<u8>) -> Packet {
    let pack = Packet::new(buff, self.transport.addr, String::new());

    let buf = serialize(&pack).unwrap();

    self.transport.send(addr, buf);

    pack
  }

  pub fn send_answer(net: &mut Network<T>, addr: &SocketAddr, buff: Vec<u8>, response_to: String) {
    let pack = Packet::new(buff, net.transport.addr, response_to);

    let buf = serialize(&pack).unwrap();

    net.transport.send(addr, buf);
  }
}
