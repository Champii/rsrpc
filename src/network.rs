use std::thread;
use std::net::{ SocketAddr };
use bincode::{ serialize };
use std::sync::{ Arc };

use super::proto::Packet;
use super::transport::*;
use super::utils::to_socket_addr;

pub struct ServerCallback {
  pub closure: Box<Fn(Vec<u8>) -> Vec<u8>>,
}

impl ServerCallback {
  pub fn new(closure: Box<Fn(Vec<u8>) -> Vec<u8>>) -> ServerCallback {
    ServerCallback {
      closure,
    }
  }

  pub fn new_empty() -> ServerCallback {
    Self::new(Box::new(|a| { a }))
  }
}

unsafe impl Send for ServerCallback {}
unsafe impl Sync for ServerCallback {}

pub struct Network<T: Transport + Clone> {
  pub running: bool,
  pub transport: T,
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

impl<T: 'static +  Transport + Clone + Send + Sync> Network<T> {
  pub fn new_default(addr: &SocketAddr) -> Network<T> {
    let t = T::new(addr);

    Self::new(t, ServerCallback::new_empty())
  }

  pub fn new(transport: T, callback: ServerCallback) -> Network<T> {
    Network {
      transport: transport,
      running: true,
      callback: Arc::new(callback),
    }
  }

  pub fn async_read_loop(net: Network<T>) -> thread::JoinHandle<()> {
    let net = net.clone();

    trace!("Starting async read loop");

    thread::spawn(|| {
      Self::run_read_thread(net);
    })
  }

  fn run_read_thread(net: Network<T>) {
    let mut t = net.transport.clone();

    loop {
      match t.recv() {
        Ok(buff) => (net.callback.closure)(buff),
        Err(_) => break,
      };
    }
  }

  pub fn set_callback(&mut self, callback: ServerCallback) {
    self.callback = Arc::new(callback);
  }

  pub fn send(&mut self, addr: &SocketAddr, buff: Vec<u8>) -> Packet {
    let pack = Packet::new(buff, self.transport.get_addr(), String::new());

    let buf = serialize(&pack).unwrap();

    self.transport.send(addr, buf);

    pack
  }

  pub fn send_answer(net: &mut Network<T>, addr: &SocketAddr, buff: Vec<u8>, response_to: String) {
    let pack = Packet::new(buff, net.transport.get_addr(), response_to);

    let buf = serialize(&pack).unwrap();

    net.transport.send(addr, buf);
  }

  pub fn close(net: &mut Network<T>) {
    T::close(&mut net.transport.clone());
  }
}
