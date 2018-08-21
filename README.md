# RSRPC

### Rust Simple RPC

## Info

Largely inspired by Tarpc and Bifrost

Under development

Basic synchronous RPC system in UDP by default, but with multiple transport solutions.

## Usage

```rust
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

fn main() {
  let mut server = HelloServer::listen("127.0.0.1:3000");

  let mut client = ServiceClient::connect("127.0.0.1:3000");

  client.eq(42, 43); //  return false
  client.hello("world".to_string()); // return "hello world"

  client.close();
  server.close();

  Server::wait_thread(server);
}
```
