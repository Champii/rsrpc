use bincode::serialize;
use hex::encode;
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, Hash, Clone)]
pub struct PacketHeader {
  pub sender: SocketAddr,
  pub date: u64,
  pub msg_hash: String,
  pub response_to: String,
}

impl PacketHeader {
  pub fn new(sender: SocketAddr, response_to: String) -> PacketHeader {
    PacketHeader {
      sender,
      date: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64,
      msg_hash: String::from(""),
      response_to,
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Hash, Clone)]
pub struct Packet {
  pub header: PacketHeader,
  pub data: Vec<u8>,
}

impl Packet {
  pub fn new(data: Vec<u8>, sender: SocketAddr, response_to: String) -> Packet {
    let mut pack = Packet {
      header: PacketHeader::new(sender, response_to),
      data,
    };

    pack._hash();

    pack
  }

  pub fn _hash(&mut self) {
    let serie = serialize(self).unwrap();
    let mut sha = Sha256::default();
    sha.input(serie.as_slice());
    self.header.msg_hash = encode(sha.result().as_slice());
  }
}
