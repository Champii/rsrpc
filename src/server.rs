use std::thread;
use std::sync::{ Arc, Mutex };

use super::network::Network;
use super::proto::Packet;
use super::transport::Transport;
use super::utils::Mutexed;

#[derive(Clone)]
pub struct Interceptor<T>(pub Arc<Mutexed<Arc<Fn(T) -> T>>>);

impl<T> Interceptor<T> {
  pub fn new() -> Interceptor<T> {
    Interceptor(Arc::new(Mutexed::new(Arc::new(|a| { a }))))
  }

  pub fn set(&mut self, cb: Arc<Fn(T) -> T>) {
    let mut guard = self.0.mutex.lock().unwrap();

    *guard = cb;
  }

  pub fn run(&self, t: T) -> T {
    (self.0.get())(t)
  }
}

pub struct Server<T: Transport> {
  pub network: Network<T>,
  pub handle: Option<thread::JoinHandle<()>>,
  pub interceptor: Interceptor<Packet>,
}

impl<T: 'static + Transport> Server<T> {
  pub fn new(net: Network<T>) -> Server<T> {
    Server {
      network: net,
      handle: None,
      interceptor: Interceptor::new(),
    }
  }

  fn wait_thread(server: Server<T>) {
    trace!("Server: Waiting for thread...");

    server.handle.unwrap().join().unwrap();
  }

  pub fn close(self) {
    debug!("Server: Closing...");

    Network::<T>::close(&mut self.network.clone());

    Self::wait_thread(self);

    info!("Server: Closed");
  }

  pub fn set_interceptor(&mut self, cb: Arc<Fn(Packet) -> Packet>) {
    self.interceptor.set(cb);
  }
}


