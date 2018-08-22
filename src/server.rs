use std::thread;
use std::sync::{ Arc };

use super::network::Network;
use super::proto::Packet;
use super::transport::Transport;
use super::interceptor::Interceptor;

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

  pub fn wait_thread(server: Server<T>) {
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


