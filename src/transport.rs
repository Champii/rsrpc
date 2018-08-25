use std::net::{SocketAddr, UdpSocket};
use std::io::{Error, ErrorKind};
use std::sync::{ Arc, Mutex };

pub struct UdpTransport {
  pub addr: SocketAddr,
  pub socket: Option<UdpSocket>,
  pub running: Arc<Mutex<bool>>,
}

unsafe impl Send for UdpTransport {}
unsafe impl Sync for UdpTransport {}

impl UdpTransport {
  fn set_running(&mut self, running: bool) {
    let mut guard = self.running.lock().unwrap();

    *guard = running;
  }

  fn get_running(&mut self) -> bool {
    let mut guard = self.running.lock().unwrap();

    (*guard).clone()
  }
}

impl Clone for UdpTransport {
  fn clone(&self) -> Self {
    let socket = match self.socket.as_ref() {
      Some(s) => Some(s.try_clone().unwrap()),
      None => None,
    };

    UdpTransport {
      socket,
      addr: self.addr.clone(),
      running: self.running.clone(),
    }
  }
}

pub trait Transport: Sync + Sized + Clone + Send {
  fn new(addr: &SocketAddr) -> Self;
  fn get_addr(&self) -> SocketAddr;
  fn listen(&mut self);
  fn send(&mut self, addr: &SocketAddr, data: Vec<u8>);
  fn recv(&mut self) -> Result<(Vec<u8>, SocketAddr), Error>;
  fn close(&mut self);
}

impl Transport for UdpTransport {
  fn new(addr: &SocketAddr) -> UdpTransport {
    UdpTransport {
      addr: addr.clone(),
      socket: None,
      running: Arc::new(Mutex::new(false)),
    }
  }

  fn listen(&mut self) {
    let socket = UdpSocket::bind(self.addr).unwrap();

    socket.set_read_timeout(Some(std::time::Duration::from_millis(100))).unwrap();

    self.socket = Some(socket);

    self.set_running(true);
  }

  fn get_addr(&self) -> SocketAddr {
    // TODO: return the address from the socket instead
    self.addr
    // self.socket.as_ref().map(|socket | socket.local_addr().unwrap()).unwrap()
  }

  fn send(&mut self, addr: &SocketAddr, buff: Vec<u8>) {
    if let Some(s) = self.socket.as_ref() {
      s.send_to(buff.as_slice(), addr).unwrap();
    }

    trace!("Sent {} to {}", buff.len(), addr);
  }

  fn recv(&mut self) -> Result<(Vec<u8>, SocketAddr), Error> {
    let mut buff = [0; 2048];

    if !self.get_running() {
      return Err(Error::new(ErrorKind::Other, "Not running"));
    }

    match self.socket.as_ref().unwrap().recv_from(&mut buff) {
      Ok((amount, from)) => {
        trace!("Read {} from {}", amount, from);

        if amount == 0 && !self.get_running() {
          trace!("Forced read 0 and not running");

          Err(Error::new(ErrorKind::Other, "Read 0"))
        } else {
          Ok((buff[..amount].to_vec(), from))
        }
      }
      Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
        self.recv()
      }
      Err(e) => {
        Err(e)
      }
    }
  }

  fn close(&mut self) {
    self.set_running(false);

    drop(self.socket.take().unwrap());
  }
}
