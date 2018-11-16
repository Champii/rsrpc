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
  fn simple_test_udp() {
    let mut server = Foo::listen_udp("127.0.0.1:3000");
    let mut client = Foo::connect_udp("127.0.0.1:3000").unwrap();

    assert_eq!(
      client.hello("test".to_string()),
      Ok(Ok("hello test".to_string()))
    );
    assert_eq!(client.eq(42, 43), Ok(Ok(false)));

    client.close();
    server.close();
  }

  #[test]
  fn simple_test_tcp() {
    let mut server = Foo::listen_tcp("127.0.0.1:3000");
    let mut client = Foo::connect_tcp("127.0.0.1:3000").unwrap();

    assert_eq!(
      client.hello("test".to_string()),
      Ok(Ok("hello test".to_string()))
    );
    assert_eq!(client.eq(42, 43), Ok(Ok(false)));

    client.close();
    server.close();
  }

  #[test]
  fn explicit_transport_type() {
    let mut server = Foo::listen_with::<Foo::UdpTransport>("127.0.0.1:3001");
    let mut client = Foo::connect_with::<Foo::UdpTransport>("127.0.0.1:3001").unwrap();

    assert_eq!(client.eq(42, 42), Ok(Ok(true)));
    assert_eq!(
      client.hello("test2".to_string()),
      Ok(Ok("hello test2".to_string()))
    );

    client.close();
    server.close();
  }

  #[test]
  fn explicit_tcp_transport_type() {
    let mut server = Foo::listen_with::<Foo::TcpTransport>("127.0.0.1:3002");
    let mut client = Foo::connect_with::<Foo::TcpTransport>("127.0.0.1:3002").unwrap();

    assert_eq!(client.eq(42, 42), Ok(Ok(true)));
    assert_eq!(
      client.hello("test2".to_string()),
      Ok(Ok("hello test2".to_string()))
    );

    client.close();
    server.close();
  }

  #[test]
  fn explicit_provided_network() {
    let mut net1 =
      Network::<Foo::UdpTransport>::new_default(&super::super::to_socket_addr("127.0.0.1:3003"));
    let mut net2 =
      Network::<Foo::UdpTransport>::new_default(&super::super::to_socket_addr("127.0.0.1:3003"));

    net1.listen();
    net2.connect().unwrap();

    let mut server = Foo::listen_with_network(net1);
    let mut client = Foo::connect_with_network(net2);

    assert_eq!(client.eq(42, 43), Ok(Ok(false)));
    assert_eq!(
      client.hello("test3".to_string()),
      Ok(Ok("hello test3".to_string()))
    );

    client.close();
    server.close();
  }

  #[test]
  fn explicit_provided_tcp_network() {
    let mut net1 =
      Network::<Foo::TcpTransport>::new_default(&super::super::to_socket_addr("127.0.0.1:3004"));
    let mut net2 =
      Network::<Foo::TcpTransport>::new_default(&super::super::to_socket_addr("127.0.0.1:3004"));

    net1.listen();
    net2.connect().unwrap();

    let mut server = Foo::listen_with_network(net1);
    let mut client = Foo::connect_with_network(net2);

    assert_eq!(client.eq(42, 43), Ok(Ok(false)));
    assert_eq!(
      client.hello("test3".to_string()),
      Ok(Ok("hello test3".to_string()))
    );

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
    let mut server = Foo::listen_udp("127.0.0.1:3010");
    let mut client = Foo::connect_udp("127.0.0.1:3010").unwrap();

    assert_eq!(
      client.hello("test4".to_string()),
      Ok(Ok("hello test4".to_string()))
    );
    assert_eq!(client.eq(42, 43), Ok(Ok(false)));

    client.close();
    server.close();
  }

  #[test]
  fn simple_test_bar() {
    let mut server = Bar::listen_tcp("127.0.0.1:3011");
    let mut client = Bar::connect_tcp("127.0.0.1:3011").unwrap();

    assert_eq!(
      client.hello2("test5".to_string()),
      Ok(Ok("hello 2 test5".to_string()))
    );
    assert_eq!(client.neq2(42, 43), Ok(Ok(true)));

    client.close();
    server.close();
  }
}

mod context {
  #[allow(unused_imports)]
  use std::net::SocketAddr;

  #[allow(unused_imports)]
  use std::sync::{Arc, Mutex};

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
  fn test_udp() {
    let mut server = Foo::listen_udp("127.0.0.1:3020");
    let mut client = Foo::connect_udp("127.0.0.1:3020").unwrap();

    assert_eq!(client.inc(1), Ok(Ok(1)));
    assert_eq!(client.inc(2), Ok(Ok(3)));
    assert_eq!(client.inc(3), Ok(Ok(6)));

    client.close();
    server.close();
  }

  #[test]
  fn test_tcp() {
    let mut server = Foo::listen_tcp("127.0.0.1:3020");
    let mut client = Foo::connect_tcp("127.0.0.1:3020").unwrap();

    assert_eq!(client.inc(1), Ok(Ok(1)));
    assert_eq!(client.inc(2), Ok(Ok(3)));
    assert_eq!(client.inc(3), Ok(Ok(6)));

    client.close();
    server.close();
  }
}

mod duplex {
  #[allow(unused_imports)]
  use super::super::network::Network;
  #[allow(unused_imports)]
  use std::net::SocketAddr;

  service! {
    Foo {
      fn hello(&mut self, name: String) -> String {
        format!("hello {}", name)
      }
    }
  }

  #[test]
  fn test_duplex() {
    let server = Foo::Duplex::listen("127.0.0.1:3030");
    let mut client = Foo::Duplex::connect("127.0.0.1:3030");

    assert_eq!(
      client.hello("test".to_string()),
      Ok(Ok("hello test".to_string()))
    );

    drop(server);
    drop(client);
    Foo::Duplex::close();
  }
}
