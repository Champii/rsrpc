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
pub struct Network {
  pub running: bool,
  pub transport: UdpTransport,
  pub callback: Arc<ServerCallback>,
}

impl Network {
  pub fn new(transport: UdpTransport, callback: ServerCallback) -> Network {
    Network {
      transport: transport,
      running: true,
      callback: Arc::new(callback),
    }
  }

  pub fn listen(net: Network) -> thread::JoinHandle<()> {
    let net = net.clone();

    thread::spawn(move || {
      let mut net = net;

      Self::run_read_thread(&mut net);
    })
  }

  pub fn run_read_thread(net: &mut Network) {
    loop {
      match net.transport.recv() {
        Ok(buff) => (net.callback.closure)(buff),
        Err(_) => break,
      };
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

  pub fn send_answer(net: &mut Network, addr: &SocketAddr, buff: Vec<u8>, response_to: String) {
    let pack = Packet::new(buff, net.transport.addr, response_to);

    let buf = serialize(&pack).unwrap();

    net.transport.send(addr, buf);
  }
}
