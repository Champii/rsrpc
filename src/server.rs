use std::thread;

use super::network::Network;
use super::transport::Transport;

pub struct Server<T: Transport> {
  pub network: Network<T>,
  pub handle: Option<thread::JoinHandle<()>>,
}

impl<T: 'static + Transport> Server<T> {
  pub fn wait_thread(server: Server<T>) {
    server.handle.unwrap().join().unwrap();
  }

  pub fn close(&mut self) {
    debug!("Server: Closing");
    Network::<T>::close(&mut self.network);
  }
}
