use std::net::SocketAddr;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

mod tcp_transport;
mod udp_transport;

pub use self::tcp_transport::TcpTransport;
pub use self::udp_transport::UdpTransport;

pub trait Transport: Sync + Sized + Clone + Send {
  fn new(addr: &SocketAddr) -> Self;
  fn get_addr(&self) -> SocketAddr;
  fn listen(&mut self);
  fn connect(&mut self) -> Result<(), String>;
  fn send(&mut self, addr: &SocketAddr, data: Vec<u8>) -> bool;
  fn get_recv(&mut self) -> Arc<Mutex<Receiver<(Vec<u8>, SocketAddr)>>>;
  fn is_running(&mut self) -> bool;
  fn close(&mut self);
}
