use super::proto::Packet;

pub struct ServerCallback {
  pub closure: Box<Fn(Packet) -> Packet>,
}

impl ServerCallback {
  pub fn new(closure: Box<Fn(Packet) -> Packet>) -> ServerCallback {
    ServerCallback {
      closure,
    }
  }

  pub fn new_empty() -> ServerCallback {
    Self::new(Box::new(|a| { a }))
  }
}

unsafe impl Send for ServerCallback {}
unsafe impl Sync for ServerCallback {}
