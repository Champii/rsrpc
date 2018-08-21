# RSRPC

### Rust Simple RPC

## Info

Largely inspired by Tarpc and Bifrost

Under development

Basic synchronous RPC system in UDP by default, but with multiple transport solutions.

## Usage

By default RSRPC uses UDP as transport system. See [Transport](#transport)

```rust
service! {
  rpc hello(name: String) -> String;
  rpc eq(s1: u8, s2: u8) -> bool;
}

pub struct HelloClient;
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

fn main() {
  let mut server = HelloServer::listen("127.0.0.1:3000");

  let mut client = HelloClient::connect("127.0.0.1:3000");

  let _ = client.eq(42, 43); //  return false

  let _ = client.hello("world".to_string()); // return "hello world"

  client.close();

  server.close();

  Server::wait_thread(server);
}
```

## Transport

You can chose the Transport to connect with :

```rust
  let server = HelloServer::listen_with::<UdpTransport>("127.0.0.1:3000");

  let client = HelloClient::connect_with::<UdpTransport>("127.0.0.1:3000");
```

Actualy only UdpTransport is implemented but a TcpTransport is in the pipe.
