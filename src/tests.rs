
mod tests {
  #[allow(unused_imports)]
  use std::net::SocketAddr;

  #[allow(unused_imports)]
  use std::sync::Arc;

  #[allow(unused_imports)]
  use super::super::network::Network;

  service! {
    Foo {
      fn hello(name: String) -> String {
        format!("hello {}", name)
      }

      fn eq(s1: u8, s2: u8) -> bool {
        s1 == s2
      }
    }
  }

  #[test]
  fn simple_test() {
    env_logger::init();

    let server = Foo::listen("127.0.0.1:3000");
    let mut client = Foo::connect("127.0.0.1:3000");

    assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());
    assert_eq!(client.eq(42, 43), false);

    client.close();
    server.close();
  }

  #[test]
  fn interceptor() {
    let mut server = Foo::listen("127.0.0.1:3001");
    let mut client = Foo::connect("127.0.0.1:3001");

    server.set_interceptor(Arc::new(|pack| {
      // todo
      pack
    }));

    assert_eq!(client.eq(42, 43), false);
    assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());

    client.close();
    server.close();
  }

  #[test]
  fn explicit_transport_type() {
    let server = Foo::listen_with::<Foo::UdpTransport>("127.0.0.1:3002");
    let mut client = Foo::connect_with::<Foo::UdpTransport>("127.0.0.1:3002");

    assert_eq!(client.eq(42, 43), false);
    assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());

    client.close();
    server.close();
  }

  #[test]
  fn explicit_provided_network() {
    let mut net = Network::new_default(&"127.0.0.1:3003".parse::<SocketAddr>().unwrap());
    let mut net2 = Network::<Foo::UdpTransport>::new_default(&"127.0.0.1:3004".parse::<SocketAddr>().unwrap());

    let server = Foo::listen_with_network(&mut net);
    let mut client = Foo::connect_with_network(&mut net2, net.transport.get_addr());

    assert_eq!(client.eq(42, 43), false);
    assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());

    client.close();
    server.close();
  }
}

mod multi_service_tests {
  #[allow(unused_imports)]
  use std::net::SocketAddr;

  #[allow(unused_imports)]
  use std::sync::Arc;

  #[allow(unused_imports)]
  use super::super::network::Network;

  service! {
    Foo {
      fn hello(name: String) -> String {
        format!("hello {}", name)
      }

      fn eq(s1: u8, s2: u8) -> bool {
        s1 == s2
      }
    }
    Bar {
      fn hello(name: String) -> String {
        format!("hello 2 {}", name)
      }

      fn neq(s1: u8, s2: u8) -> bool {
        s1 != s2
      }
    }
  }

  #[test]
  fn simple_test_foo() {
    let server = Foo::listen("127.0.0.1:3010");
    let mut client = Foo::connect("127.0.0.1:3010");

    assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());
    assert_eq!(client.eq(42, 43), false);

    client.close();
    server.close();
 }

  #[test]
  fn simple_test_bar() {
    let server = Bar::listen("127.0.0.1:3011");
    let mut client = Bar::connect("127.0.0.1:3011");

    assert_eq!(client.hello("moi_lol".to_string()), "hello 2 moi_lol".to_string());
    assert_eq!(client.neq(42, 43), true);

    client.close();
    server.close();
  }
}

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