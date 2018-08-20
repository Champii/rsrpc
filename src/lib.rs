#![feature(async_await, await_macro, pin, arbitrary_self_types, futures_api)]
#[macro_use] extern crate serde_derive;

extern crate serde;
extern crate serde_bytes;
extern crate bincode;
extern crate futures;
extern crate byteorder;
extern crate sha2;
extern crate hex;
extern crate tokio_core;
extern crate pin_utils;
#[macro_use]
extern crate lazy_static;

mod network;
mod proto;
mod transport;
mod async_response_matcher;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use byteorder::{ByteOrder, LittleEndian};

pub fn prepend_u64 (num: u64, vec: Vec<u8>) -> Vec<u8> {
  let mut s_id_vec = [0u8; 8].to_vec();

  LittleEndian::write_u64(&mut s_id_vec, num);

  let data_iter = s_id_vec.into_iter().chain(vec.into_iter());

  data_iter.collect()
}

pub fn extract_u64_head(vec: Vec<u8>) -> (u64, Vec<u8>) {
  let num = LittleEndian::read_u64(&vec);

  let vec: Vec<u8> = vec.into_iter().skip(8).collect();

  (num, vec)
}

pub fn hash_ident(id: &str) -> usize {
  let id = id.to_string();

  let mut hasher = DefaultHasher::new();

  hasher.write(&id.into_bytes());
  hasher.finish() as usize
}

#[macro_export]
macro_rules! as_is {
  ($x:expr) => ( $x );
}

#[macro_export]
macro_rules! hash_ident {
  ($x:ident) => ( $crate::hash_ident(stringify!($x)) )
}

// this macro expansion design took credits from tarpc by Google Inc.
#[macro_export]
macro_rules! service {
  (
    $(
      $(#[$attr:meta])*
      rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ) $(-> $out:ty)* $(| $error:ty)*;
    )*
  ) => {
    service! {{
      $(
        $(#[$attr])*
        rpc $fn_name( $( $arg : $in_ ),* ) $(-> $out)* $(| $error)*;
      )*
    }}
  };
  (
    {
      $(#[$attr:meta])*
      rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ); // No return, no error

      $( $unexpanded:tt )*
    }
    $( $expanded:tt )*
  ) => {
    service! {
      { $( $unexpanded )* }

      $( $expanded )*

      $(#[$attr])*
      rpc $fn_name( $( $arg : $in_ ),* ) -> () | ();
    }
  };
  (
    {
      $(#[$attr:meta])*
      rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ) -> $out:ty; //return, no error

      $( $unexpanded:tt )*
    }
    $( $expanded:tt )*
  ) => {
    service! {
      { $( $unexpanded )* }

      $( $expanded )*

      $(#[$attr])*
      rpc $fn_name( $( $arg : $in_ ),* ) -> $out | ();
    }
  };
  (
    {
      $(#[$attr:meta])*
      rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ) | $error:ty; //no return, error

      $( $unexpanded:tt )*
    }
    $( $expanded:tt )*
  ) => {
    service! {
      { $( $unexpanded )* }

      $( $expanded )*

      $(#[$attr])*
      rpc $fn_name( $( $arg : $in_ ),* ) -> () | $error;
    }
  };
  (
    {
      $(#[$attr:meta])*
      rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ) -> $out:ty | $error:ty; //return, error

      $( $unexpanded:tt )*
    }
    $( $expanded:tt )*
  ) => {
    service! {
      { $( $unexpanded )* }

      $( $expanded )*

      $(#[$attr])*
      rpc $fn_name( $( $arg : $in_ ),* ) -> $out | $error;
    }
  };
  (
    {} // all expanded
    $(
      $(#[$attr:meta])*
      rpc $fn_name:ident ( $( $arg:ident : $in_:ty ),* ) -> $out:ty | $error:ty;
    )*
  ) => {

    use std::thread;
    use std::net::SocketAddr;
    use std::collections::HashMap;
    use std::sync::{ Mutex };
    use futures::channel::oneshot;
    use futures::executor::block_on;
    use bincode::{ serialize, deserialize };

    use $crate::network::*;
    use $crate::transport::*;
    use $crate::async_response_matcher::*;


    lazy_static! {
      pub static ref MATCHER: Mutex<AsyncResponseMatcher> = Mutex::new(AsyncResponseMatcher::new());
    }

    pub fn to_socket_addr(s: &str) -> SocketAddr {
      match s.parse::<SocketAddr>() {
        Ok(addr) => addr,
        Err(e) => {
          panic!("Invalid address: {}, {}", s, e);
        },
      }
    }

    pub struct Server {
      network: Network,
    }

    pub trait Service {
      $(
        fn $fn_name($($arg:$in_),*) -> $out;
      )*

      fn dispatch(data: Vec<u8>) -> Vec<u8> {
        let (func_id, body) = super::extract_u64_head(data);
        let mut toto: HashMap<usize, Box<Fn() -> Vec<u8>>> = HashMap::new();

        $(
          toto.insert(hash_ident!($fn_name), Box::new(|| -> Vec<u8> {
            let ($($arg,)*) : ($($in_,)*) = $crate::bincode::deserialize(&body).unwrap();
            $crate::bincode::serialize(&Self::$fn_name($($arg,)*)).unwrap()
          }));
        )*;

        let tocall = toto.get(&(func_id as usize)).unwrap();

        let res = tocall();

        res
      }

      fn listen(addr: &str) -> Server {
        let transport = UdpTransport::new(&to_socket_addr(addr));

        // empty ServerCallback as we need a reference to network to define it later. See Network::set_callback(...);
        let mut network = Network::new(transport, ServerCallback { closure: Box::new(|d| { d }) });

        let net_c = network.clone();

        let server = Server {
          network: network.clone(),
        };

        let cb = ServerCallback {
          closure: Box::new(move |buff| {
            let mut net = net_c.clone();

            let pack: Packet = deserialize(&buff).unwrap();

            let res = Self::dispatch(pack.data);

            Network::send_answer(&mut net, &pack.header.sender, serialize(&res).unwrap(), pack.header.msg_hash.clone());

            buff
          }),
        };

        network.set_callback(cb);

        Network::listen(network.clone());

        server
      }
    }


    pub struct ServiceClient {
      serv_addr: SocketAddr,
      network: Network,
    }

    impl ServiceClient {
      fn connect(addr: &str) -> ServiceClient {
        let mut addr = to_socket_addr(addr.clone());

        let serv_addr = addr.clone();

        addr.set_port(0);

        let transport = UdpTransport::new(&addr);

        let network = Network::new(transport, ServerCallback {
          closure: Box::new(move |data| {
            let pack: Packet = deserialize(&data).unwrap();

            thread::spawn(move || {
              let mut test = MATCHER.lock().unwrap();

              AsyncResponseMatcher::resolve(&mut *test, pack.header.response_to.clone(), pack.data.clone());
            });

            data
          }),
        });

        Network::listen(network.clone());

        ServiceClient {
          serv_addr,
          network,
        }
      }

      $(
        fn $fn_name(&mut self, $($arg:$in_),*) -> $out {
          let (tx1, rx1) = oneshot::channel::<Vec<u8>>();

          let req_data = ($($arg,)*);
          let req_data_bytes = $crate::bincode::serialize(&req_data).unwrap();
          let req_bytes = $crate::prepend_u64(hash_ident!($fn_name) as u64, req_data_bytes);

          let pack = self.network.send(&self.serv_addr, req_bytes);

          thread::spawn(move || {
            let mut test = MATCHER.lock().unwrap();

            let lol = &mut *test;

            lol.add(pack.header.msg_hash, tx1);
          });

          let mut res = Vec::new();

          block_on(async {
            res = await!(rx1).unwrap();
          });

          $crate::bincode::deserialize(&res).unwrap()
        }
      )*
    }
  }
}

mod simple_test {
  use super::proto::*;

  type Hash = String;

  service! {
    rpc hello(name: String) -> String;
    rpc eq(s1: u8, s2: u8) -> bool;
  }

  pub struct HelloServer;

  impl Service for HelloServer {
    fn hello(name: String) -> String {
      format!("hello {}", name)
    }

    fn eq(s1: u8, s2: u8) -> bool {
      s1 == s2
    }
  }

  #[test]
  fn simple_test() {
    let h1 = thread::spawn(move || {
      HelloServer::listen("127.0.0.1:3000");
    });

    let h2 = thread::spawn(move || {
      let mut client = ServiceClient::connect("127.0.0.1:3000");

      assert_eq!(client.eq(42, 42), true);
      assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());
    });

    h1.join().unwrap();
    h2.join().unwrap();
  }
}
