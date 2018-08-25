
mod tests {
  #[allow(unused_imports)]
  use std::net::SocketAddr;

  #[allow(unused_imports)]
  use std::sync::Arc;

  #[allow(unused_imports)]
  use super::super::network::Network;

  service! {
    Foo {
      fn hello(&mut self, name: String) -> String {
        format!("hello {}", name)
      }

      fn eq(&mut self, s1: u8, s2: u8) -> bool {
        s1 == s2
      }
    }
  }

  #[test]
  fn simple_test() {

    let mut server = Foo::listen("127.0.0.1:3000");
    let mut client = Foo::connect("127.0.0.1:0", "127.0.0.1:3000");

    assert_eq!(client.hello("test".to_string()), "hello test".to_string());
    assert_eq!(client.eq(42, 43), false);

    client.close();
    server.close();
  }

  // #[test]
  // fn interceptor() {
  //   let mut server = Foo::listen("127.0.0.1:3001");
  //   let mut client = Foo::connect("127.0.0.1:3001");

  //   server.set_interceptor(Arc::new(|pack| {
  //     // todo
  //     pack
  //   }));

  //   assert_eq!(client.eq(42, 43), false);
  //   assert_eq!(client.hello("test".to_string()), "hello test".to_string());

  //   client.close();
  //   server.close();
  // }

  #[test]
  fn explicit_transport_type() {
    let mut server = Foo::listen_with::<Foo::UdpTransport>("127.0.0.1:3002");
    let mut client = Foo::connect_with::<Foo::UdpTransport>("127.0.0.1:0", "127.0.0.1:3002");

    assert_eq!(client.eq(42, 42), true);
    assert_eq!(client.hello("test2".to_string()), "hello test2".to_string());

    client.close();
    server.close();
  }

  #[test]
  fn explicit_provided_network() {
    let mut net1 = Network::<Foo::UdpTransport>::new_default(&super::super::to_socket_addr("127.0.0.1:3003"));
    let mut net2 = Network::<Foo::UdpTransport>::new_default(&super::super::to_socket_addr("127.0.0.1:3004"));

    net1.listen();
    net2.listen();

    let addr = net1.transport.get_addr();

    let mut server = Foo::listen_with_network(net1);
    let mut client = Foo::connect_with_network(net2, &addr);

    assert_eq!(client.eq(42, 43), false);
    assert_eq!(client.hello("test3".to_string()), "hello test3".to_string());

    client.close();
    server.close();
  }
}

mod multi_service {
  #[allow(unused_imports)]
  use std::net::SocketAddr;

  #[allow(unused_imports)]
  use std::sync::Arc;

  #[allow(unused_imports)]
  use super::super::network::Network;

  service! {
    Foo {
      fn hello(&mut self, name: String) -> String {
        format!("hello {}", name)
      }

      fn eq(&mut self, s1: u8, s2: u8) -> bool {
        s1 == s2
      }
    }
    // TODO: Beware of name collision !
    Bar {
      fn hello2(&mut self, name: String) -> String {
        format!("hello 2 {}", name)
      }

      fn neq2(&mut self, s1: u8, s2: u8) -> bool {
        s1 != s2
      }
    }
  }

  #[test]
  fn simple_test_foo() {
    let mut server = Foo::listen("127.0.0.1:3010");
    let mut client = Foo::connect("127.0.0.1:0", "127.0.0.1:3010");

    assert_eq!(client.hello("test4".to_string()), "hello test4".to_string());
    assert_eq!(client.eq(42, 43), false);

    client.close();
    server.close();
 }

  #[test]
  fn simple_test_bar() {
    let mut server = Bar::listen("127.0.0.1:3011");
    let mut client = Bar::connect("127.0.0.1:0", "127.0.0.1:3011");

    assert_eq!(client.hello2("test5".to_string()), "hello 2 test5".to_string());
    assert_eq!(client.neq2(42, 43), true);

    client.close();
    server.close();
  }
}


mod context {
  #[allow(unused_imports)]
  use std::net::SocketAddr;

  #[allow(unused_imports)]
  use std::sync::{ Arc, Mutex };

  #[allow(unused_imports)]
  use super::super::network::Network;

  service! {
    Foo {
      let ctx: Arc<Mutex<u8>>;

      fn inc(&mut self, n: u8) -> u8 {
        let mut guard = self.ctx.lock().unwrap();

        *guard += n;

        *guard
      }
    }
  }

  #[test]
  fn test() {
    let mut server = Foo::listen("127.0.0.1:3020");
    let mut client = Foo::connect("127.0.0.1:0", "127.0.0.1:3020");

    assert_eq!(client.inc(1), 1);
    assert_eq!(client.inc(2), 3);
    assert_eq!(client.inc(3), 6);

    client.close();
    server.close();
  }
}

mod duplex {
  #[allow(unused_imports)]
  use std::net::SocketAddr;
  #[allow(unused_imports)]
  use super::super::network::Network;

  service! {
    Foo {
      fn hello(&mut self, name: String) -> String {
        format!("hello {}", name)
      }
    }
  }

  #[test]
  fn test_duplex() {
    env_logger::init();

    let mut duplex = Foo::Duplex::new("127.0.0.1:3030");

    let mut server = duplex.listen();
    let mut client = duplex.connect("127.0.0.1:3030");

    assert_eq!(client.hello("test".to_string()), "hello test".to_string());

    duplex.close(server, &mut vec![client]);
  }
}