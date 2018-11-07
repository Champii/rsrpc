# RSRPC

### Rust Simple RPC

## Info

Largely inspired by Tarpc and Bifrost

Under development and currently not stable.

Basic synchronous RPC system in UDP by default, but with multiple transport solutions.

## Index

* [Usage](#usage)
* [Transport](#transport)
* [Network](#network)
* [Multi-services](#multi-services)
* [Duplex](#duplex)
* [Stateful-Context](#stateful-context)
* [Plugins](#plugins)

## Usage

By default RSRPC uses UDP as transport system. See [Transport](#transport)

```rust
service! {
  // Here is the name of your service
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
  // The server listening
  let mut server = Foo::listen("127.0.0.1:3000");

  // The client listening to :3001 and connecting to :3000
  let mut client = Foo::connect("127.0.0.1:3001", "127.0.0.1:3000");


  // returns "hello world"
  let _ = client.hello("world".to_string());

  // returns true
  let _ = client.ping();


  // You can also use `.wait()` if you prefere to wait for another event to quit
  client.close();
  server.close();
}
```

Every exemple show a server and a client in the same instance for brievety. They can obviously be separated. However if you want to have a single shared socket for a client and a server on the same instance, see the [Duplex](#duplex) section.

## Transport

You can chose the Transport to connect with :

```rust
  let server = Foo::listen_with::<UdpTransport>("127.0.0.1:3000");

  let client = Foo::connect_with::<UdpTransport>("127.0.0.1:3001", "127.0.0.1:3000");
```

Actualy only UdpTransport is implemented but a TcpTransport is in the pipe.

## Network

You can chose the Network to connect with :

```rust
  use rsrpc::network::Network;
  use rsrpc::transport::UdpTransport;

  let addr1 = rsrpc::to_socket_addr("127.0.0.1:3000");
  let addr2 = rsrpc::to_socket_addr("127.0.0.1:3001");

  // You must call listen if you instantiate a Network by yourself
  let mut net = Network::<UdpTransport>::new_default(&addr1).listen();
  let mut net2 = Network::<UdpTransport>::new_default(&addr2).listen();

  let server = Foo::listen_with_network(&mut net);

  // The second argument is the address to connect to
  let client = Foo::connect_with_network(&mut net2, net.transport.get_addr());
```

This may be usefull to have some servers or clients to share the same binded socket.

See the [Duplex](#duplex) section to see a more conveignant way to make a server and a client to share the same socket.

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

## Duplex

If you want to have a single binded socket (Probably to connect server to server) you can use the Duplex object like so :

```rust
  // You must listen first, as this will initiate the shared network object
  let server = Foo::Duplex::listen("127.0.0.1:3000");

  // Connect to another server through the same socket
  let mut client = Foo::Duplex::connect("127.0.0.1:7777");

  // You must destroy every reference to the shared network before closing or waiting
  drop(server);
  drop(client);

  // This will close and wait for the thread. You can also only wait with `wait()`
  Foo::Duplex::close();
```


## Stateful context

You can declare some variables to a service in order to keep a context :

```rust
service! {
  Foo {
    // You must explicitly give a type
    let hello: String = "hello !".to_string();

    // if no assignation, we take `Default::default()`
    let i: Arc<Mutex<u8>>;

    // This service increments a mutexed `u8` by the amount given as parameter
    fn inc(&mut self, n: u8) -> u8 {
      let mut guard = self.i.lock().unwrap();

      *guard += n;

      *guard
    }
  }
}

fn main() {
  // The context is accessible through the `Server` object
  let server = Foo::listen("127.0.0.1:3000");

  // This context is an `Arc<Mutex<T>>`
  println!("Say hello: {}", server.context.lock().unwrap().hello);
}
```

## Plugins

You can add some plugins at runtime to catch incoming and outgoing packets to append some logic sequentialy.

You can then modify the packet before the send or after the receive, in order to implement incremental protocol. (or just to log what's happening. It's up to you :D)

Each plugin is called in the order of declaration for `on_send()`, and in the reverse order for `on_recv()`

You have to make a struct to implement the `Wrapper` trait

```rust
use rsrpc::{ Wrapper, Plugins };

// You can keep a context here
struct TestWrapper;

impl Wrapper for TestWrapper {
  // Called before each packet send
  fn on_send(&self, pack: &Packet) -> Packet {
    /* ... */
  }

  // Called after each packet received
  fn on_recv(&self, pack: &Packet) -> Packet {
    /* ... */
  }
}

fn main() {
  /* ... */

  Plugins::add(TestWrapper);

  /* ... */
}
```


## TODO

- Move back AsyncResponseMatcher into Client instead of Network
- Error management
- Futures management with a `send_async` call (`struct AsyncClient;` ?)
- Remove interceptor as it can be replaced by `trait Wrapper` and `Plugins`
- Doc
  - Server::wait_thread
  - Client::wait_thread
