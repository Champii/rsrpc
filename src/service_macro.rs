

// this macro expansion design took credits from tarpc by Google Inc.
#[macro_export]
macro_rules! service {
  (
    $(
      $(#[$attr:meta])*
      rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ) $(-> $out:ty)* $(| $error:ty)*;
    )*
  ) => {
    service! {{
      $(
        $(#[$attr])*
        rpc $fn_name( $( $arg : $in_ ),* ) $(-> $out)* $(| $error)*;
      )*
    }}
  };
  (
    {
      $(#[$attr:meta])*
      rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ); // No return, no error

      $( $unexpanded:tt )*
    }
    $( $expanded:tt )*
  ) => {
    service! {
      { $( $unexpanded )* }

      $( $expanded )*

      $(#[$attr])*
      rpc $fn_name( $( $arg : $in_ ),* ) -> () | ();
    }
  };
  (
    {
      $(#[$attr:meta])*
      rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ) -> $out:ty; //return, no error

      $( $unexpanded:tt )*
    }
    $( $expanded:tt )*
  ) => {
    service! {
      { $( $unexpanded )* }

      $( $expanded )*

      $(#[$attr])*
      rpc $fn_name( $( $arg : $in_ ),* ) -> $out | ();
    }
  };
  (
    {
      $(#[$attr:meta])*
      rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ) | $error:ty; //no return, error

      $( $unexpanded:tt )*
    }
    $( $expanded:tt )*
  ) => {
    service! {
      { $( $unexpanded )* }

      $( $expanded )*

      $(#[$attr])*
      rpc $fn_name( $( $arg : $in_ ),* ) -> () | $error;
    }
  };
  (
    {
      $(#[$attr:meta])*
      rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ) -> $out:ty | $error:ty; //return, error

      $( $unexpanded:tt )*
    }
    $( $expanded:tt )*
  ) => {
    service! {
      { $( $unexpanded )* }

      $( $expanded )*

      $(#[$attr])*
      rpc $fn_name( $( $arg : $in_ ),* ) -> $out | $error;
    }
  };
  (
    {} // all expanded
    $(
      $(#[$attr:meta])*
      rpc $fn_name:ident ( $( $arg:ident : $in_:ty ),* ) -> $out:ty | $error:ty;
    )*
  ) => {
    pub use $crate::proto::Packet;
    pub use $crate::server::{ Server, Interceptor };
    pub use $crate::transport::{ Transport, UdpTransport };
    pub use $crate::client::Client;
    pub use $crate::utils::to_socket_addr;
    pub use std::sync::{ Arc, Mutex };

    pub trait RpcServer {
      $(
        fn $fn_name($($arg:$in_),*) -> $out;
      )*

      fn dispatch(pack: $crate::Packet) -> Vec<u8> {
        // let pack = Self::before_all(pack);

        let (func_id, body) = $crate::extract_u64_head(pack.data.clone());
        let mut hmap: $crate::HashMap<usize, Box<Fn() -> Vec<u8>>> = $crate::HashMap::new();

        $(
          hmap.insert($crate::hash_ident!($fn_name), Box::new(|| -> Vec<u8> {
            let ($($arg,)*) : ($($in_,)*) = $crate::bincode::deserialize(&body).unwrap();

            debug!("Server: {} > {}", &pack.header.sender, stringify!($fn_name));

            let call_res = &Self::$fn_name($($arg,)*);

            debug!("Server: {} < {}", &pack.header.sender, stringify!($fn_name));

            $crate::bincode::serialize(call_res).unwrap()
          }));
        )*;

        let tocall = hmap.get(&(func_id as usize)).unwrap();

        tocall()
      }

      fn listen(addr: &str) -> $crate::Server<$crate::UdpTransport> {
        Self::listen_with::<UdpTransport>(addr)
      }

      fn listen_with<T: 'static +  Transport>(addr: &str) -> $crate::Server<T> {

        let mut network = $crate::Network::new_default(&to_socket_addr(addr));

        Self::listen_with_network(&mut network)
      }

      fn listen_with_network<T: 'static +  Transport>(net: &mut $crate::Network<T>) -> $crate::Server<T> {
        info!("Server: Listening {}", net.transport.get_addr());

        let mut net = net.clone();

        let net_c = net.clone();

        let mut server = $crate::Server::new(net.clone());

        let interceptor = server.interceptor.clone();

        net.set_callback($crate::ServerCallback {
          closure: Box::new(move |buff| {
            let mut net = net_c.clone();

            let pack: $crate::Packet = $crate::deserialize(&buff).unwrap();

            let pack = interceptor.run(pack);

            let res = Self::dispatch(pack.clone());

            $crate::Network::send_answer(&mut net, &pack.header.sender, res, pack.header.msg_hash.clone());

            buff
          }),
        });

        server.handle = Some($crate::Network::async_read_loop(net.clone()));

        server
      }
    }

    pub trait RpcClient {
      fn connect(addr: &str) -> Client<UdpTransport> {
        Self::connect_with::<UdpTransport>(addr)
      }

      fn connect_with<T: 'static +  Transport>(addr: &str) -> Client<T> {
        let mut addr = to_socket_addr(addr.clone());

        let serv_addr = addr.clone();

        addr.set_port(0);

        let mut network = $crate::Network::new_default(&addr);

        let mut client = Self::connect_with_network(&mut network);

        client.serv_addr = serv_addr;

        client
      }

      fn connect_with_network<T: 'static +  Transport>(network: &mut $crate::Network<T>) -> Client<T> {
        info!("Client: Connecting {}", network.transport.get_addr());

        let mut network = network.clone();

        // network.set_callback($crate::ServerCallback {
        //   closure: Box::new(move |data| {
        //     let pack: $crate::Packet = $crate::deserialize(&data).unwrap();

        //     $crate::thread::spawn(move || {
        //       let mut guard = $crate::service_macro::MATCHER.lock().unwrap();

        //       $crate::AsyncResponseMatcher::resolve(&mut *guard, pack.header.response_to.clone(), pack.data.clone());
        //     });

        //     data
        //   }),
        // });

        Client {
          serv_addr: network.transport.get_addr(),
          network: network.clone(),
          handle: Some($crate::Network::async_read_loop(network.clone())),
        }
      }
    }

    // #[allow(dead_code)]
    // impl<T: 'static +  Transport> ServiceClient<T> {
    pub trait ServiceClient {
      fn get_serv_addr(&mut self) -> $crate::SocketAddr;
      fn send(&mut self, addr: &$crate::SocketAddr, data: Vec<u8>) -> Vec<u8>;

      $(
        fn $fn_name(&mut self, $($arg:$in_),*) -> $out {
          // let (tx1, rx1) = $crate::oneshot::channel::<Vec<u8>>();

          let req_data = ($($arg,)*);
          let req_data_bytes = $crate::bincode::serialize(&req_data).unwrap();
          let req_bytes = $crate::prepend_u64($crate::hash_ident!($fn_name) as u64, req_data_bytes);
          let addr = self.get_serv_addr();

          debug!("Client: {} < {}", addr, stringify!($fn_name));

          let res = self.send(&addr, req_bytes);

          // $crate::thread::spawn(move || {
          //   let mut guard = $crate::service_macro::MATCHER.lock().unwrap();

          //   let matcher = &mut *guard;

          //   matcher.add(pack.header.msg_hash, tx1);
          // }).join().unwrap();

          // let mut res = Vec::new();

          // $crate::block_on(async {
          //   res = await!(rx1).unwrap();
          // });

          debug!("Client: {} > {}", addr, stringify!($fn_name));

          $crate::bincode::deserialize(&res).unwrap()
        }
      )*
    }

    impl<T: 'static + Transport> ServiceClient for Client<T> {
      fn get_serv_addr(&mut self) -> $crate::SocketAddr {
        self.serv_addr.clone()
      }

      fn send(&mut self, addr: &$crate::SocketAddr, data: Vec<u8>) -> Vec<u8> {
        self.network.send(addr, data)
      }
    }
  }
}
