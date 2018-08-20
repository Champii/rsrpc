use std::net::{SocketAddr, UdpSocket};

pub struct UdpTransport {
  pub addr: SocketAddr,
  pub socket: UdpSocket,
}

impl UdpTransport {
  pub fn new(addr: &SocketAddr) -> UdpTransport {
    let socket = UdpSocket::bind(addr).unwrap();

    UdpTransport {
      addr: socket.local_addr().unwrap(),
      socket: socket,
    }
  }
}

impl Clone for UdpTransport {
  fn clone(&self) -> UdpTransport {
    let socket2 = self.socket.try_clone().unwrap();

    UdpTransport {
      addr: self.addr.clone(),
      socket: socket2,
    }
  }
}

unsafe impl Send for UdpTransport {}

pub trait Transport {
  fn new_udp(addr: &SocketAddr) -> UdpTransport {
    UdpTransport::new(addr)
  }
  fn send(&mut self, addr: &SocketAddr, data: Vec<u8>);
  fn recv(&mut self) -> Vec<u8>;
}

impl Transport for UdpTransport {
  fn send(&mut self, addr: &SocketAddr, buff: Vec<u8>) {
    self.socket.send_to(buff.as_slice(), addr).unwrap();
  }

  fn recv(&mut self) -> Vec<u8> {
    let mut buff = [0; 2048];

    let toto = self.socket.recv_from(&mut buff);

    match toto {
      Ok((amount, _)) => {
        buff[..amount].to_vec()
      }
      Err(e) => {
        println!("ERR {:?}", e);
        buff.to_vec()
      }
    }
  }
}