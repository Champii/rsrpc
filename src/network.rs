use std::thread;
use std::net::{ SocketAddr };
use bincode::{ serialize };
use std::sync::{ Arc, Mutex };
use std::io::{Error, ErrorKind};

use super::proto::Packet;
use super::transport::*;
use super::plugins::*;
use super::utils::*;
use super::async_response_matcher::AsyncResponseMatcher;
use super::server_callback::ServerCallback;
use super::oneshot::channel;

lazy_static! {
  pub static ref MATCHER: super::Mutex<super::AsyncResponseMatcher> = super::Mutex::new(super::AsyncResponseMatcher::new());
}

#[derive(Clone)]
pub struct Network<T: Transport + Clone> {
  pub transport: T,
  pub callback: Mutexed<ServerCallback>,
  pub handle: Option<Arc<thread::JoinHandle<()>>>,
}

// impl<T: Transport + Clone> Clone for Network<T> {
//   fn clone(&self) -> Self {
//     Network {
//       transport: self.transport.clone(),
//       callback: self.callback.clone(),
//       handle: self.handle.clone(),
//     }
//   }
// }

impl<T: 'static +  Transport + Clone + Send + Sync> Network<T> {
  pub fn new_default(addr: &SocketAddr) -> Network<T> {
    let t = T::new(addr);

    Self::new(t, ServerCallback::new_empty())
  }

  pub fn new(transport: T, callback: ServerCallback) -> Network<T> {
    Network {
      transport: transport,
      callback: Mutexed::new(callback),
      handle: None,
    }
  }

  pub fn listen(&mut self) -> &mut Network<T> {
    self.transport.listen();

    let net = self.clone();

    trace!("Starting async read loop");

    self.handle = Some(Arc::new(thread::spawn(|| {
      Self::run_read_thread(net);
    })));

    self
  }

  fn run_read_thread(net: Network<T>) {
    let mut t = net.transport.clone();
    let net = net.clone();


    loop {
      match t.recv() {
        Ok((buff, from)) => {
          let mut pack: Packet = super::deserialize(&buff).unwrap();

          pack = PLUGINS.get().run_on_recv(pack.clone());

          let pack_c = pack.clone();

          thread::spawn(move || {
            let mut guard = MATCHER.lock().unwrap();

            AsyncResponseMatcher::resolve(&mut *guard, pack.header.response_to.clone(), pack.data.clone());
          });

          (net.callback.get().closure)(pack_c, from);
        },
        Err(e) => {
          if e.kind() != ErrorKind::Other {
            error!("Error read {}", e);
          }

          break;
        }
      };
    }
  }

  pub fn set_callback(&mut self, callback: ServerCallback) {
    self.callback.set(callback);
  }

  // pub fn send(&mut self, addr: &SocketAddr, buff: Vec<u8>) -> Result<Vec<u8>, futures::channel::oneshot::Canceled> {
  pub fn send(&mut self, addr: &SocketAddr, buff: Vec<u8>) -> Vec<u8> {
    let (tx1, rx1) = channel::<Vec<u8>>();

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
      match await!(rx1) {
        Ok(r) => res = r,
        Canceled => warn!("Canceled call"),
      };
    });

    res
  }

  pub fn send_answer(net: &mut Network<T>, addr: &SocketAddr, buff: Vec<u8>, response_to: String) {
    let mut pack = Packet::new(buff, net.transport.get_addr(), response_to);

    pack = PLUGINS.get().run_on_send(pack.clone());

    let buf = serialize(&pack).unwrap();

    net.transport.send(addr, buf);
  }

  pub fn wait(&mut self) {
    if let Some(handle) = self.handle.take() {
      let h = Arc::try_unwrap(handle).unwrap();

      h.join().unwrap();
    }
  }

  pub fn close(&mut self) {
    self.transport.close();

    self.set_callback(ServerCallback::new_empty());

    MATCHER.lock().unwrap().close();
  }
}
