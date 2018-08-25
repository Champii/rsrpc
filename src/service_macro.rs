#[macro_export]
macro_rules! service {
  // Base Rule
  (
    $(
      $service_name:ident {
        $(
          let $var:ident : $type_:ty $(= $default:expr)* ;
        )*
        $(
          fn $fn_name:ident(&mut $self_:ident , $( $arg:ident : $in_:ty ),* ) $(-> $out:ty)* $(| $error:ty)* $b:block
        )*
      }
    )*
  ) => {
    service! {{
      $(
        $service_name {
          {
            {
              $(let $var : $type_ $(= $default)* ;)*
            }
          }
          {{
            $(fn $fn_name(&mut $self_ , $( $arg : $in_ ),* ) $(-> $out)* $(| $error)* $b)*
          }}
        }
      )*
    }}
  };

  // Variable no default value
  (
    {
      $(
        $service_name:ident {
          {
            {
              let $var:ident : $type_:ty ;
              $($unexpanded_var:tt)*
            }
            $($expanded_var:tt)*
          }
          {{$($unexpanded_fn:tt)*}}
        }
      )*
    }
  ) => {
    service! {{
      $(
        $service_name {
          {
            { $($unexpanded_var)* }
            $($expanded_var)*
            let $var : $type_  = Default::default() ;
          }
          {{$($unexpanded_fn)*}}
        }
      )*
    }}
  };

  // Variable with default value
  (
    {
      $(
        $service_name:ident {
          {
            {
              let $var:ident : $type_:ty = $default:expr ;
              $($unexpanded_var:tt)*
            }
            $($expanded_var:tt)*
          }
          {{$($unexpanded_fn:tt)*}}
        }
      )*
    }
  ) => {
    service! {{
      $(
        $service_name {
          {
            { $($unexpanded_var)* }
            $($expanded_var)*
            let $var : $type_  = $default ;
          }
          {{$($unexpanded_fn)*}}
        }
      )*
    }}
  };


  // Func with no return
  (
    {
      $(
        $service_name:ident {
          {
            {}
            $($expanded_var:tt)*
          }
          {
            {
              fn $fn_name:ident(&mut $self_:ident , $( $arg:ident : $in_:ty ),* ) $b:block
              $($unexpanded_fn:tt)*
            }
            $($expanded_fn:tt)*
          }
        }
      )*
    }
  ) => {
    service! {{
      $(
        $service_name {
          {
            {}
            $($expanded_var)*
          }
          {
            {$($unexpanded_fn:tt)*}
            $($expanded_fn:tt)*
            fn $fn_name:ident(&mut $self_:ident , $( $arg:ident : $in_:ty ),* ) -> () | () $b:block
          }
        }
      )*
    }}
  };

  // Func with return
  (
    {
      $(
        $service_name:ident {
          {
            {}
            $($expanded_var:tt)*
          }
          {
            {
              fn $fn_name:ident(&mut $self_:ident , $( $arg:ident : $in_:ty ),* ) -> $out:ty $b:block
              $($unexpanded_fn:tt)*
            }
            $($expanded_fn:tt)*
          }
        }
      )*
    }
  ) => {
    service! {{
      $(
        $service_name {
          {
            {}
            $($expanded_var)*
          }
          {
            { $($unexpanded_fn)* }
            $($expanded_fn)*
            fn $fn_name(&mut $self_ , $( $arg : $in_ ),* ) -> $out | () $b
          }
        }
      )*
    }}
  };

  // Func with error
  (
    {
      $(
        $service_name:ident {
          {
            {}
            $($expanded_var:tt)*
          }
          {
            {
              fn $fn_name:ident(&mut $self_:ident , $( $arg:ident : $in_:ty ),* ) | $err:ty $b:block
              $($unexpanded_fn:tt)*
            }
            $($expanded_fn:tt)*
          }
        }
      )*
    }
  ) => {
    service! {{
      $(
        $service_name {
          {
            {}
            $($expanded_var)*
          }
          {
            {$($unexpanded_fn)*}
            $($expanded_fn)*
            fn $fn_name(&mut $self_ , $( $arg : $in_ ),* ) -> () | $err $b
          }
        }
      )*
    }}
  };

  // Func with return and error
  (
    {
      $(
        $service_name:ident {
          {
            {}
            $($expanded_var:tt)*
          }
          {
            {
              fn $fn_name:ident(&mut $self_:ident , $( $arg:ident : $in_:ty ),* ) -> $out:ty | $err:ty $b:block
              $($unexpanded_fn:tt)*
            }
            $($expanded_fn:tt)*
          }
        }
      )*
    }
  ) => {
    service! {{
      $(
        $service_name {
          {
            {}
            $($expanded_var)*
          }
          {
            {$($unexpanded_fn)*}
            $($expanded_fn)*
            fn $fn_name(&mut $self_ , $( $arg : $in_ ),* ) -> $out | $err $b
          }
        }
      )*
    }}
  };

  // Final form
  (
    {
      $(
        $service_name:ident {
          {
            {}
            $(let $var:ident : $type_:ty = $default:expr ;)*
          }
          {
            {}
            $(fn $fn_name:ident(&mut $self_:ident , $( $arg:ident : $in_:ty ),* ) -> $out:ty | $error:ty $block:block)*
          }
        }
      )*
    }
  ) => {
    pub use $crate::transport::{ Transport, UdpTransport };

    $(
      #[allow(non_snake_case)]
      pub mod $service_name {
        pub use $crate::transport::{ Transport, UdpTransport };
        use $crate::utils::to_socket_addr;
        use std::sync::{ Arc, Mutex };

        #[allow(unused)]
        #[derive(Clone)]
        pub struct $service_name {
          $(pub $var: $type_,)*
        }

        impl $service_name {
          pub fn new() -> $service_name {
            $service_name {
              $($var: $default,)*
            }
          }
        }

        pub trait ServiceTrait {
          $(
            fn $fn_name(&mut $self_, $($arg:$in_),*) -> $out;
          )*

          fn dispatch(ctx: &mut $service_name, pack: $crate::Packet) -> Vec<u8> {
            let (func_id, body) = $crate::extract_u64_head(pack.data.clone());

            // fixme: This is dirty as hell, we redefine a HashMap each time dispatch is called !
            let mut hmap: $crate::HashMap<usize, Box<Fn() -> Vec<u8>>> = $crate::HashMap::new();

            $(
              hmap.insert($crate::hash_ident!($fn_name), Box::new(|| -> Vec<u8> {
                let mut ctx_c = ctx.clone();

                let ($($arg,)*) : ($($in_,)*) = $crate::bincode::deserialize(&body).unwrap();

                debug!("Server: {} > {}", &pack.header.sender, stringify!($fn_name));

                let call_res = &ctx_c.$fn_name($($arg,)*);

                debug!("Server: {} < {}", &pack.header.sender, stringify!($fn_name));

                $crate::bincode::serialize(call_res).unwrap()
              }));
            )*;


            let tocall = hmap.get(&(func_id as usize)).unwrap();

            tocall()
          }
        }

        #[allow(unused)]
        #[derive(Clone)]
        pub struct Client<T: Transport> {
          pub serv_addr: $crate::SocketAddr,
          pub network: $crate::Network<T>,
        }

        impl<T: 'static + Transport> Client<T> {
          #[allow(unused)]
          fn wait(&mut self) {
            self.network.wait();
          }

          #[allow(unused)]
          pub fn close(&mut self) {
            debug!("Client: Closing...");

            self.network.close();

            self.wait();

            info!("Client: Closed");
          }
          #[allow(unused)]
          fn get_serv_addr(&mut self) -> $crate::SocketAddr {
            self.serv_addr.clone()
          }

          #[allow(unused)]
          fn send(&mut self, addr: &$crate::SocketAddr, data: Vec<u8>) -> Vec<u8> {
            self.network.send(addr, data)
          }
          // fn send(&mut self, addr: &$crate::SocketAddr, data: Vec<u8>) -> Result<Vec<u8>, &str> {
          //   match self.network.send(addr, data) {
          //     Ok(r) => Ok(r),
          //     Canceled => Err("Canceled"),
          //   }
          // }

          $(

            #[allow(unused)]
            pub fn $fn_name(&mut self, $($arg:$in_),*) -> $out {
              let req_data = ($($arg,)*);
              let req_data_bytes = $crate::bincode::serialize(&req_data).unwrap();
              let req_bytes = $crate::prepend_u64($crate::hash_ident!($fn_name) as u64, req_data_bytes);
              let addr = self.get_serv_addr();

              debug!("Client: {} < {}", addr, stringify!($fn_name));

              let res = self.send(&addr, req_bytes);

              // $crate::bincode::deserialize(&res)
              match res.len() == 0 {
                false => {
                  debug!("Client: {} > {}", addr, stringify!($fn_name));

                  $crate::bincode::deserialize(&res).unwrap()
                },
                true => {
                  error!("Error client send for {}", stringify!($fn_name));

                  Default::default()
                }
              }
            }
          )*
        }

        #[derive(Clone)]
        pub struct Server<T: Transport> {
          pub network: $crate::Network<T>,
          pub context: Arc<Mutex<$service_name>>,
        }

        impl<T: 'static + Transport> Server<T> {
          pub fn new(net: $crate::Network<T>) -> Server<T> {
            Server {
              network: net,
              context: Arc::new(Mutex::new($service_name::new())),
            }
          }

          #[allow(unused)]
          pub fn wait(&mut self) {
            trace!("Server: Waiting for thread...");

            self.network.wait();
          }

          #[allow(unused)]
          pub fn close(&mut self) {
            debug!("Server: Closing...");

            self.network.close();

            self.wait();

            info!("Server: Closed");
          }

          // #[allow(unused)]
          // pub fn set_interceptor(&mut self, cb: Arc<Fn($crate::Packet) -> $crate::Packet>) {
          //   self.interceptor.set(cb);
          // }
        }

        pub struct Duplex {
          network: $crate::Network<UdpTransport>,
        }

        impl Duplex {
          pub fn new(addr: &str) -> Duplex {
            Duplex {
              network: $crate::Network::new_default(&$crate::utils::to_socket_addr(addr)),
            }
          }

          pub fn listen(&mut self) -> Server<UdpTransport> {
            self.network.listen();

            listen_with_network(self.network.clone())
          }

          pub fn connect(&mut self, addr: &str) -> Client<UdpTransport> {
            connect_with_network(self.network.clone(), &addr.parse::<$crate::SocketAddr>().unwrap())
          }

          pub fn close(&mut self, server: Server<UdpTransport>, clients: &mut Vec<Client<UdpTransport>>) {
            debug!("Server: Closing...");

            drop(server);

            for i in (0..clients.len()) {
              let c = clients.pop();

              drop(c);
            }

            self.network.close();

            trace!("Server: Waiting for thread...");

            self.network.wait();

            info!("Server: Closed");
          }
        }

        // pub trait RpcDuplex {
        //   $(
        //     fn $fn_name($($arg:$in_),*) -> $($out)*;
        //   )*

        //   // fn new() -> Duplex {
        //     // Duplex::new()
        //   // }

        // }

        impl ServiceTrait for $service_name {
          $(
            fn $fn_name(&mut $self_, $( $arg : $in_ ),* ) -> $out $block
          )*
        }

        #[allow(unused)]
        pub fn connect(addr: &str, serv_addr: &str) -> Client<UdpTransport> {
          connect_with::<UdpTransport>(addr, serv_addr)
        }

        pub fn connect_with<T: 'static +  Transport>(addr: &str, serv_addr: &str) -> Client<T> {
          let mut network = $crate::Network::new_default(&to_socket_addr(addr));

          network.listen();

          connect_with_network(network, &to_socket_addr(serv_addr))
        }

        pub fn connect_with_network<T: 'static +  Transport>(network: $crate::Network<T>, serv_addr: &$crate::SocketAddr) -> Client<T> {
          let mut net = network;

          info!("Client: Listening {}", net.transport.get_addr());

          Client {
            serv_addr: serv_addr.clone(),
            network: net,
          }
        }

        #[allow(unused)]
        pub fn listen(addr: &str) -> Server<$crate::UdpTransport> {
          listen_with::<UdpTransport>(addr)
        }

        #[allow(unused)]
        pub fn listen_with<T: 'static +  Transport>(addr: &str) -> Server<T> {

          let mut network = $crate::Network::new_default(&to_socket_addr(addr));

          network.listen();

          listen_with_network(network)
        }

        #[allow(unused)]
        pub fn listen_with_network<T: 'static +  Transport>(net: $crate::Network<T>) -> Server<T> {
          info!("Server: Listening {}", net.transport.get_addr());
          let mut net = net;

          // net.listen();

          let net_c = net.clone();

          let mut server = Server::new(net.clone());

          let mut context = server.context.clone();

          server.network.set_callback($crate::ServerCallback {
            closure: Arc::new(move |pack, from| {
              if pack.header.response_to.len() == 0 {

                let mut net = net_c.clone();

                let mut context = context.clone();

                let mut guard = context.lock().unwrap();

                let res = $service_name::dispatch(&mut *guard, pack.clone());

                $crate::Network::send_answer(&mut net, &from, res, pack.header.msg_hash.clone());
              }

              pack
            }),
          });

          server
        }
      }
    )*
  }
}
