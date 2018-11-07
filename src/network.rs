use std::thread;
use std::time::Duration;
use std::net::{ SocketAddr };
use bincode::{ serialize };
use std::sync::{ Arc };
use std::io::{ ErrorKind };
use futures::select;

use super::timer::Timer;
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
  pub plugins: Plugins,
  pub callback: Mutexed<ServerCallback>,
  pub handle: Option<Arc<thread::JoinHandle<()>>>,
}

impl<T: 'static +  Transport + Clone + Send + Sync> Network<T> {
  pub fn new_default(addr: &SocketAddr) -> Network<T> {
    let t = T::new(addr);

    Self::new(t, ServerCallback::new_empty())
  }

  pub fn new(transport: T, callback: ServerCallback) -> Network<T> {
    Network {
      transport: transport,
      plugins: Plugins::new(),
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

          let mut plugins = net.plugins.clone();

          pack = plugins.run_on_recv(pack.clone());

          let pack_c = pack.clone();

          {
            let mut guard = MATCHER.lock().unwrap();

            AsyncResponseMatcher::resolve(&mut *guard, pack.header.response_to.clone(), pack.data.clone());
          }

          (net.callback.get().closure)(pack_c, from);
        },
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
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

  pub fn send(&mut self, addr: &SocketAddr, buff: Vec<u8>) -> Result<Vec<u8>, String> {
    let (tx1, mut rx1) = channel::<Vec<u8>>();

    let mut err_rx = Timer::new(Duration::from_secs(1), "Timeout".to_string());

    let pack = Packet::new(buff, self.transport.get_addr(), String::new());

    let mut pack_c = pack.clone();

    let mut transport = self.transport.clone();

    let addr_c = addr.clone();

    let mut plugins = self.plugins.clone();

    {
      let addr_c = addr_c.clone();

      let mut guard = MATCHER.lock().unwrap();

      let matcher = &mut *guard;

      matcher.add(pack_c.header.msg_hash.clone(), tx1);

      pack_c = plugins.run_on_send(pack_c.clone());

      let buf = serialize(&pack_c).unwrap();

      transport.send(&addr_c, buf);
    }

    let mut res_vec = Ok(Vec::new());

    super::block_on(async {
      select! {
        rx1 => {
          match rx1 {
            Ok(r) => res_vec = Ok(r),
            _ => warn!("Canceled call"),
          };
        },
        err_rx => {
          match err_rx {
            Ok(err) => {
              error!("Error sending to {} : {}", addr, err);

              {
                let mut guard = MATCHER.lock().unwrap();

                let matcher = &mut *guard;

                AsyncResponseMatcher::remove(matcher, pack_c.header.msg_hash.clone());
              }

              res_vec = Err(err);
            },
            _ => warn!("Canceled error callback"),
          };
        },
      };
    });

    res_vec
  }

  pub fn send_answer(net: &mut Network<T>, addr: &SocketAddr, buff: Vec<u8>, response_to: String) {
    let mut pack = Packet::new(buff, net.transport.get_addr(), response_to);

    pack = net.plugins.run_on_send(pack.clone());

    let buf = serialize(&pack).unwrap();

    net.transport.send(addr, buf);
  }

  pub fn wait(&mut self) {
    if let Some(handle) = self.handle.take() {
      match Arc::try_unwrap(handle) {
        Ok(h) => h.join().unwrap(),
        Err(a) => warn!("Not waiting: multiple references ({}) to network stay.", Arc::strong_count(&a)),
      };
    }
  }

  pub fn close(&mut self) {
    self.transport.close();

    self.set_callback(ServerCallback::new_empty());

    MATCHER.lock().unwrap().close();
  }
}
