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
  pub fn new(transport: T, callback: ServerCallback) -> Network<T> {
    Network {
      transport: transport,
      running: true,
      callback: Arc::new(callback),
    }
  }

  pub fn listen(net: Network<T>) -> thread::JoinHandle<()> {
    let toto = net.clone();

    thread::spawn(|| {
      // let mut toto = toto.clone();

      Self::run_read_thread(toto);
    })
  }

  pub fn run_read_thread(net: Network<T>) {
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
    // net.transport.close();
  }
}
