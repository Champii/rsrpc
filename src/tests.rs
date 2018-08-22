
mod simple_test {
  service! {
    rpc hello(name: String) -> String;
    rpc eq(s1: u8, s2: u8) -> bool;
  }

  #[allow(dead_code)]
  pub struct Foo;

  impl RpcService for Foo {
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

    let mut server = Service::listen("127.0.0.1:3000");

    let mut client = Service::connect("127.0.0.1:3000");

    assert_eq!(client.eq(42, 43), false);
    assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());

    client.close();
    server.close();
  }
}

// mod interceptor {
//   service! {
//     rpc hello(name: String) -> String;
//     rpc eq(s1: u8, s2: u8) -> bool;
//   }

//  #[allow(dead_code)]
//   pub struct Foo;

//   impl RpcService for Foo {
//     fn hello(name: String) -> String {
//       format!("hello {}", name)
//     }

//     fn eq(s1: u8, s2: u8) -> bool {
//       s1 == s2
//     }
//   }

//   #[test]
//   fn test() {
//     let mut server = Service::listen("127.0.0.1:3001");

//     server.set_interceptor(Arc::new(|pack| {
//       println!("TEST TEST {:?}", pack);
//       pack
//     }));

//     let mut client = Service::connect("127.0.0.1:3001");

//     assert_eq!(client.eq(42, 43), false);
//     assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());

//     client.close();
//     server.close();
//   }
// }

// mod explicit_transport_type {
//   service! {
//     rpc hello(name: String) -> String;
//     rpc eq(s1: u8, s2: u8) -> bool;
//   }

//   #[allow(dead_code)]
//   pub struct Foo;

//   impl RpcService for Foo {
//     fn hello(name: String) -> String {
//       format!("hello {}", name)
//     }

//     fn eq(s1: u8, s2: u8) -> bool {
//       s1 == s2
//     }
//   }

//   #[test]
//   fn test() {
//     let mut server = Service::listen_with::<UdpTransport>("127.0.0.1:3002");

//     let mut client = Service::connect_with::<UdpTransport>("127.0.0.1:3002");

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
//   pub struct Foo;

//   impl RpcService for Foo {
//     fn hello(name: String) -> String {
//       format!("hello {}", name)
//     }

//     fn eq(s1: u8, s2: u8) -> bool {
//       s1 == s2
//     }
//   }

//   #[test]
//   fn test2() {
//     let mut net = Network::<UdpTransport>::new_default(&"127.0.0.1:3003".parse::<SocketAddr>().unwrap());
//     let mut net2 = Network::<UdpTransport>::new_default(&"127.0.0.1:3004".parse::<SocketAddr>().unwrap());

//     let server = Service::listen_with_network(&mut net);

//     let mut client = Service::connect_with_network(&mut net2, net.transport.get_addr());

//     assert_eq!(client.eq(42, 43), false);
//     assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());

//     server.close();
//     client.close();
//   }
// }

// mod duplex {
//   #[allow(unused_imports)]
//   use std::net::SocketAddr;
//   #[allow(unused_imports)]
//   use super::super::network::Network;

//   service! {
//     rpc hello(name: String) -> String;
//     rpc eq(s1: u8, s2: u8) -> bool;
//   }

//   #[allow(dead_code)]
//   pub struct HelloDuplex;

//   impl RpcDuplex for HelloDuplex {
//     fn hello(name: String) -> String {
//       format!("hello {}", name)
//     }

//     fn eq(s1: u8, s2: u8) -> bool {
//       s1 == s2
//     }
//   }

//   #[test]
//   fn test() {
//     let duplex = HelloDuplex::new();

//     let server = duplex.listen("127.0.0.1:3000");
//     let client = duplex.connect("127.0.0.1:3000");

//     assert_eq!(client.eq(42, 43), false);
//     assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());

//     client.close();
//     server.close();
//   }
// }