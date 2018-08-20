use std::net::{SocketAddr, UdpSocket};
use std::io::{Error, ErrorKind};
use std::sync::{ Arc, Mutex };

pub struct UdpTransport {
  pub addr: SocketAddr,
  pub socket: UdpSocket,
  pub running: Arc<Mutex<bool>>,
}

impl UdpTransport {
  pub fn new(addr: &SocketAddr) -> UdpTransport {
    let socket = UdpSocket::bind(addr).unwrap();

    UdpTransport {
      addr: socket.local_addr().unwrap(),
      socket: socket,
      running: Arc::new(Mutex::new(true)),
    }
  }

  pub fn close(&mut self) {
    let mut guard = self.running.lock().unwrap();

    *guard = false;

    self.socket.send_to(&[], self.addr).unwrap();
  }
}

impl Clone for UdpTransport {
  fn clone(&self) -> UdpTransport {
    let socket2 = self.socket.try_clone().unwrap();

    UdpTransport {
      addr: self.addr.clone(),
      socket: socket2,
      running: self.running.clone(),
    }
  }
}

unsafe impl Send for UdpTransport {}

pub trait Transport {
  fn send(&mut self, addr: &SocketAddr, data: Vec<u8>);
  fn recv(&mut self) -> Result<Vec<u8>, Error>;
}

impl Transport for UdpTransport {
  fn send(&mut self, addr: &SocketAddr, buff: Vec<u8>) {
    self.socket.send_to(buff.as_slice(), addr).unwrap();
  }

  fn recv(&mut self) -> Result<Vec<u8>, Error> {
    let mut buff = [0; 2048];

    {
      let running = *self.running.lock().unwrap();

      if !running {
        return Err(Error::new(ErrorKind::Other, "Not running"));
      }
    }

    match self.socket.recv_from(&mut buff) {
      Ok((amount, _)) => {
        let running = *self.running.lock().unwrap();

        if amount == 0 && !running {
          Err(Error::new(ErrorKind::Other, "Read 0"))
        } else {
          Ok(buff[..amount].to_vec())
        }
      }
      Err(e) => {
        Err(e)
      }
    }
  }
}
