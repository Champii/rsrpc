# RSRPC

### Rust Simple RPC

## Info

Largely inspired by Tarpc and Bifrost

Under development

Basic synchrone RPC system in UDP by default, but with multiple transport solutions.

## Usage

```rust
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
  let h = thread::spawn(move || {
    HelloServer::listen("127.0.0.1:3001");
  });

  let h2 = thread::spawn(move || {
    let mut client = ServiceClient::connect("127.0.0.1:3001");

    let res = client.hello("world".to_string()); //returns "hello world"
    let res = client.eq(42, 43); //returns false
  });

  h.join().unwrap();
  h2.join().unwrap();
}

```
