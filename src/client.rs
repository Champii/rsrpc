use std::net::SocketAddr;
use std::thread;

use super::transport::Transport;
use super::network::Network;

pub struct Client<T: Transport> {
  pub serv_addr: SocketAddr,
  pub network: Network<T>,
  pub handle: Option<thread::JoinHandle<()>>,
}

impl<T: 'static + Transport> Client<T> {
  fn wait_thread(client: Client<T>) {
    trace!("Client: Waiting for thread...");

    client.handle.unwrap().join().unwrap();
  }

  pub fn close(self) {
    debug!("Client: Closing...");

    Network::<T>::close(&mut self.network.clone());

    Self::wait_thread(self);

    info!("Client: Closed");
  }
}
