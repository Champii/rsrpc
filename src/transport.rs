use std::net::{SocketAddr, UdpSocket};
use std::io::{Error, ErrorKind};
use std::sync::{ Arc, Mutex };

pub struct UdpTransport {
  pub addr: SocketAddr,
  pub socket: UdpSocket,
  pub running: Arc<Mutex<bool>>,
}

unsafe impl Send for UdpTransport {}
unsafe impl Sync for UdpTransport {}

impl Clone for UdpTransport {
  fn clone(&self) -> Self {
    UdpTransport {
      addr: self.addr.clone(),
      socket: self.socket.try_clone().unwrap(),
      running: self.running.clone(),
    }
  }
}

pub trait Transport: Sync + Sized + Clone + Send {
  fn new(addr: &SocketAddr) -> Self;
  fn get_addr(&self) -> SocketAddr;
  fn send(&mut self, addr: &SocketAddr, data: Vec<u8>);
  fn recv(&mut self) -> Result<Vec<u8>, Error>;
  fn close(transport: &mut Self);
}

impl Transport for UdpTransport {
  fn new(addr: &SocketAddr) -> UdpTransport {
    let socket = UdpSocket::bind(addr).unwrap();

    UdpTransport {
      addr: socket.local_addr().unwrap(),
      socket: socket,
      running: Arc::new(Mutex::new(true)),
    }
  }

  fn get_addr(&self) -> SocketAddr {
    self.addr
  }

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

  fn close(transport: &mut UdpTransport) {
    let mut guard = transport.running.lock().unwrap();

    *guard = false;

    transport.socket.send_to(&[], transport.addr).unwrap();
  }
}
