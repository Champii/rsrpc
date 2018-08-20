use std::thread;

use super::network::Network;

pub struct Server {
  pub network: Network,
  pub handle: Option<thread::JoinHandle<()>>,
}

impl Server {
  pub fn wait_thread(server: Server) {
    server.handle.unwrap().join().unwrap();
  }

  pub fn close(&mut self) {
    self.network.transport.close();
  }
}
