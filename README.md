# RSRPC

### Rust Simple RPC

## Info

Largely inspired by Tarpc and Bifrost

Under development

Basic synchronous RPC system in UDP by default, but with multiple transport solutions.

## Index

* [Usage](#usage)
* [Transport](#transport)
* [Network](#network)
* [Multi-services](#multi-services)
* [Stateful-Context](#stateful-context)
* [Plugins](#plugins)

## Usage

By default RSRPC uses UDP as transport system. See [Transport](#transport)

```rust
service! {
  Foo {
    // You must provide a `&mut self`
    fn hello(&mut self, name: String) -> String {
      format!("hello {}", name)
    }

    // If no other arguments, you must keep the ',' after 'self'
    fn ping(&mut self,) -> bool {
      true
    }
  }
}

fn main() {
  let mut server = Foo::listen("127.0.0.1:3000");
  let mut client = Foo::connect("127.0.0.1:3000");

  let _ = client.hello("world".to_string()); // return "hello world"
  let _ = client.ping();                     // return true

  client.close();
  server.close();
}
```

## Transport

You can chose the Transport to connect with :

```rust
  let server = Foo::listen_with::<UdpTransport>("127.0.0.1:3000");

  let client = Foo::connect_with::<UdpTransport>("127.0.0.1:3001");
```

Actualy only UdpTransport is implemented but a TcpTransport is in the pipe.

## Network

You can chose the Network to connect with :

```rust
  let addr1 = rsrpc::to_socket_addr("127.0.0.1:3000");
  let addr2 = rsrpc::to_socket_addr("127.0.0.1:3001");

  let mut net = Network::<Foo::UdpTransport>::new_default(&addr1);
  let mut net2 = Network::<Foo::UdpTransport>::new_default(&addr2);

  let server = Foo::listen_with_network(&mut net);
  let client = Foo::connect_with_network(&mut net2, net.transport.get_addr());
```

## Multi-Services

You can define as many services as you want:

```rust
service! {
  Foo {
    fn bar(&mut self, name: String) -> String {
      format!("hello {}", name)
    }
  }

  Bar {
    fn foo(&mut self, name: String) -> String {
      format!("hello {}", name)
    }
  }
}
```

They have their separate module to be generated in.

## Stateful context

You can declare some variables to a service in order to keep a context :

```rust
service! {
  Foo {
    // You must explicitly give a type
    let hello: String = "hello".to_string();

    // if no assignation, we take `Default::default()`
    let ctx: Arc<Mutex<u8>>;

    fn inc(&mut self, n: u8) -> u8 {
      let mut guard = self.ctx.lock().unwrap();

      *guard += n;

      *guard
    }
  }
}
```

## Plugins

You can add some plugins at runtime to catch incoming and outgoing packets to append some logic sequentialy

You have to make a struct to have the `trait Wrapper`

```rust
use rsrpc::{ Wrapper, Plugins };

struct TestWrapper;

impl Wrapper for TestWrapper {
  fn on_send(&self, pack: &Packet) -> Packet {
    /* ... */
  }

  fn on_recv(&self, pack: &Packet) -> Packet {
    /* ... */
  }
}

fn main() {
  /* ... */

  Plugins::add(HashWrapper);

  /* ... */
}
```


## TODO

- Error management
- Duplex UDP socket to have a single transport for Server and Client
- Futures management with a `send_async` call (`struct AsyncClient;` ?)
- Remove interceptor as it can be replaced by `trait Wrapper` and `Plugins`
- Doc
  - Server::wait_thread
  - Client::wait_thread
