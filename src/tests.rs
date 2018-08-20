mod simple_test {
  use std::thread;

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
  fn test() {
    let mut server = HelloServer::listen("127.0.0.1:3001");

    let mut client = ServiceClient::connect("127.0.0.1:3001");

    assert_eq!(client.eq(42, 43), false);
    assert_eq!(client.hello("moi_lol".to_string()), "hello moi_lol".to_string());

    client.close();
    server.close();

    Server::wait_thread(server);
  }
}
