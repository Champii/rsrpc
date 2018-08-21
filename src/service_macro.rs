
lazy_static! {
  pub static ref MATCHER: super::Mutex<super::AsyncResponseMatcher> = super::Mutex::new(super::AsyncResponseMatcher::new());
}

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
    pub use $crate::server::Server;

    pub fn to_socket_addr(s: &str) -> $crate::SocketAddr {
      match s.parse::<$crate::SocketAddr>() {
        Ok(addr) => addr,
        Err(e) => {
          panic!("Invalid address: {}, {}", s, e);
        },
      }
    }

    pub trait Service {
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

            trace!("Server: {} > {}", &pack.header.sender, stringify!($fn_name));

            let call_res = &Self::$fn_name($($arg,)*);

            trace!("Server: {} < {}", &pack.header.sender, stringify!($fn_name));

            $crate::bincode::serialize(call_res).unwrap()
          }));
        )*;

        let tocall = hmap.get(&(func_id as usize)).unwrap();

        tocall()
      }

      fn listen(addr: &str) -> $crate::Server {
        trace!("Server: Listening {}", addr);

        let transport = $crate::UdpTransport::new(&to_socket_addr(addr));

        // empty ServerCallback as we need a reference to network to define it later. See Network::set_callback(...);
        let mut network = $crate::Network::new(transport, $crate::ServerCallback { closure: Box::new(|d| { d }) });

        let net_c = network.clone();

        let mut server = $crate::Server {
          network: network.clone(),
          handle: None,
        };

        let cb = $crate::ServerCallback {
          closure: Box::new(move |buff| {
            let mut net = net_c.clone();

            let pack: $crate::Packet = $crate::deserialize(&buff).unwrap();

            let res = Self::dispatch(pack.clone());

            $crate::Network::send_answer(&mut net, &pack.header.sender, res, pack.header.msg_hash.clone());

            buff
          }),
        };

        network.set_callback(cb);

        server.handle = Some($crate::Network::listen(network.clone()));

        server
      }

    }

    pub struct ServiceClient {
      serv_addr: $crate::SocketAddr,
      network: $crate::Network,
    }

    impl ServiceClient {
      pub fn connect(addr: &str) -> ServiceClient {
        trace!("Client: Connecting {}", addr);

        let mut addr = to_socket_addr(addr.clone());

        let serv_addr = addr.clone();

        addr.set_port(0);

        let transport = $crate::UdpTransport::new(&addr);

        let network = $crate::Network::new(transport, $crate::ServerCallback {
          closure: Box::new(move |data| {
            let pack: $crate::Packet = $crate::deserialize(&data).unwrap();

            $crate::thread::spawn(move || {
              let mut guard = $crate::service_macro::MATCHER.lock().unwrap();

              $crate::AsyncResponseMatcher::resolve(&mut *guard, pack.header.response_to.clone(), pack.data.clone());
            });

            data
          }),
        });

        $crate::Network::listen(network.clone());

        ServiceClient {
          serv_addr,
          network,
        }
      }

      pub fn close(&mut self) {
        debug!("Client: Closing");

        self.network.transport.close();
      }

      $(
        pub fn $fn_name(&mut self, $($arg:$in_),*) -> $out {

          let (tx1, rx1) = $crate::oneshot::channel::<Vec<u8>>();

          let req_data = ($($arg,)*);
          let req_data_bytes = $crate::bincode::serialize(&req_data).unwrap();
          let req_bytes = $crate::prepend_u64($crate::hash_ident!($fn_name) as u64, req_data_bytes);

          trace!("Client: {} < {}", &self.serv_addr, stringify!($fn_name));

          let pack = self.network.send(&self.serv_addr, req_bytes);

          $crate::thread::spawn(move || {
            let mut guard = $crate::service_macro::MATCHER.lock().unwrap();

            let matcher = &mut *guard;

            matcher.add(pack.header.msg_hash, tx1);
          }).join().unwrap();

          let mut res = Vec::new();

          $crate::block_on(async {
            res = await!(rx1).unwrap();
          });

          trace!("Client: {} > {}", &self.serv_addr, stringify!($fn_name));

          $crate::bincode::deserialize(&res).unwrap()
        }
      )*
    }
  }
}
