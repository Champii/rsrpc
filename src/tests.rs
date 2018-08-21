
// mod simple_test {
//   service! {
//     rpc hello(name: String) -> String;
//     rpc eq(s1: u8, s2: u8) -> bool;
//   }

//   #[allow(dead_code)]
//   pub struct HelloClient;
//   #[allow(dead_code)]
//   pub struct HelloServer;

//   impl RpcClient for HelloClient {}

//   impl RpcServer for HelloServer {
//     fn hello(name: String) -> String {
//       format!("hello {}", name)
//     }

//     fn eq(s1: u8, s2: u8) -> bool {
//       s1 == s2
//     }
//   }

//   #[test]
//   fn test() {
//     env_logger::init();

//     let mut server = HelloServer::listen("127.0.0.1:3000");

//     let mut client = HelloClient::connect("127.0.0.1:3000");

//     assert_eq!(client.eq(42, 43), false);
//     assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());

//     client.close();
//     server.close();
//   }
// }

mod interceptor {
  service! {
    rpc hello(name: String) -> String;
    rpc eq(s1: u8, s2: u8) -> bool;
  }

  #[allow(dead_code)]
  pub struct HelloClient;
  #[allow(dead_code)]
  pub struct HelloServer;

  impl RpcClient for HelloClient {}

  impl RpcServer for HelloServer {
    fn hello(name: String) -> String {
      format!("hello {}", name)
    }

    fn eq(s1: u8, s2: u8) -> bool {
      s1 == s2
    }
  }

  #[test]
  fn test() {
    env_logger::init();

    let mut server = HelloServer::listen("127.0.0.1:3000");

    server.set_interceptor(Arc::new(|pack| {
      println!("TEST TEST {:?}", pack);
      pack
    }));

    let mut client = HelloClient::connect("127.0.0.1:3000");

    assert_eq!(client.eq(42, 43), false);
    assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());

    client.close();
    server.close();
  }
}

// mod explicit_transport_type {
//   service! {
//     rpc hello(name: String) -> String;
//     rpc eq(s1: u8, s2: u8) -> bool;
//   }

//   #[allow(dead_code)]
//   pub struct HelloClient;
//   #[allow(dead_code)]
//   pub struct HelloServer;

//   impl RpcClient for HelloClient {}

//   impl RpcServer for HelloServer {
//     fn hello(name: String) -> String {
//       format!("hello {}", name)
//     }

//     fn eq(s1: u8, s2: u8) -> bool {
//       s1 == s2
//     }
//   }

//   #[test]
//   fn test() {
//     let mut server = HelloServer::listen_with::<UdpTransport>("127.0.0.1:3001");

//     let mut client = HelloClient::connect_with::<UdpTransport>("127.0.0.1:3001");

//     assert_eq!(client.eq(42, 43), false);
//     assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());

//     client.close();
//     server.close();
//   }
// }

// mod explicit_provided_network {
//   #[allow(unused_imports)]
//   use std::net::SocketAddr;
//   #[allow(unused_imports)]
//   use super::super::network::Network;

//   service! {
//     rpc hello(name: String) -> String;
//     rpc eq(s1: u8, s2: u8) -> bool;
//   }

//   #[allow(dead_code)]
//   pub struct HelloClient;
//   #[allow(dead_code)]
//   pub struct HelloServer;

//   impl RpcClient for HelloClient {}

//   impl RpcServer for HelloServer {
//     fn hello(name: String) -> String {
//       format!("hello {}", name)
//     }

//     fn eq(s1: u8, s2: u8) -> bool {
//       s1 == s2
//     }
//   }

//   #[test]
//   fn test() {
//     env_logger::init();
//     let mut net = Network::<UdpTransport>::new_default(&"127.0.0.1:3002".parse::<SocketAddr>().unwrap());

//     let server = HelloServer::listen_with_network(&mut net);

//     let mut client = HelloClient::connect_with_network(&mut net.clone());

//     assert_eq!(client.eq(42, 43), false);
//     assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());

//     server.close();
//     client.close();
//   }
// }