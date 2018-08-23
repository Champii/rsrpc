use std::thread;
use std::net::{ SocketAddr };
use bincode::{ serialize };
use std::sync::{ Arc };

use super::proto::Packet;
use super::transport::*;
use super::plugins::*;
use super::async_response_matcher::AsyncResponseMatcher;
use super::server_callback::ServerCallback;

lazy_static! {
  pub static ref MATCHER: super::Mutex<super::AsyncResponseMatcher> = super::Mutex::new(super::AsyncResponseMatcher::new());
}

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
    let net = net.clone();

    loop {
      match t.recv() {
        Ok(buff) => {
          let mut pack: Packet = super::deserialize(&buff).unwrap();

          pack = PLUGINS.get().run_on_recv(pack.clone());

          let pack_c = pack.clone();

          thread::spawn(move || {
            let mut guard = MATCHER.lock().unwrap();

            AsyncResponseMatcher::resolve(&mut *guard, pack.header.response_to.clone(), pack.data.clone());
          });

          (net.callback.closure)(pack_c);
        },
        Err(_) => break,
      };
    }
  }

  pub fn set_callback(&mut self, callback: ServerCallback) {
    self.callback = Arc::new(callback);
  }

  pub fn send(&mut self, addr: &SocketAddr, buff: Vec<u8>) -> Vec<u8> {
    let (tx1, rx1) = super::oneshot::channel::<Vec<u8>>();

    let pack = Packet::new(buff, self.transport.get_addr(), String::new());

    let mut pack_c = pack.clone();

    let mut transport = self.transport.clone();

    let addr_c = addr.clone();

    thread::spawn(move || {
      let addr_c = addr_c.clone();

      let mut guard = MATCHER.lock().unwrap();

      let matcher = &mut *guard;

      matcher.add(pack_c.header.msg_hash.clone(), tx1);

      pack_c = PLUGINS.get().run_on_send(pack_c.clone());

      let buf = serialize(&pack_c).unwrap();

      transport.send(&addr_c, buf);
    });

    let mut res = Vec::new();

    super::block_on(async {
      res = await!(rx1).unwrap();
    });

    res
  }

  pub fn send_answer(net: &mut Network<T>, addr: &SocketAddr, buff: Vec<u8>, response_to: String) {
    let mut pack = Packet::new(buff, net.transport.get_addr(), response_to);

    pack = PLUGINS.get().run_on_send(pack.clone());

    let buf = serialize(&pack).unwrap();

    net.transport.send(addr, buf);
  }

  pub fn close(net: &mut Network<T>) {
    T::close(&mut net.transport.clone());
  }
}
