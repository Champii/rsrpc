#[macro_export]
macro_rules! service {
  (
    $(
      $service_name:ident {
        $(
          fn $fn_name:ident( $( $arg:ident : $in_:ty ),* ) $(-> $out:ty)* $(| $error:ty)* $block:block
        )*
      }
    )*
  ) => {
    pub use $crate::transport::{ Transport, UdpTransport };

    $(
      #[allow(non_snake_case)]
      pub mod $service_name {
        pub use $crate::transport::{ Transport, UdpTransport };
        use $crate::utils::to_socket_addr;
        pub use $crate::server::Server;

        #[allow(unused)]
        pub struct $service_name;

        pub trait ServiceTrait {
          $(
            fn $fn_name($($arg:$in_),*) -> $($out)*;
          )*

          fn dispatch(pack: $crate::Packet) -> Vec<u8> {
            let (func_id, body) = $crate::extract_u64_head(pack.data.clone());

            // fixme: This is dirty as hell, we redifine a HashMap each time dispatch is called !
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
        }

        #[allow(unused)]
        pub struct Client<T: Transport> {
          pub serv_addr: $crate::SocketAddr,
          pub network: $crate::Network<T>,
          pub handle: Option<$crate::thread::JoinHandle<()>>,
        }

        impl<T: 'static + Transport> Client<T> {
          #[allow(unused)]
          fn wait_thread(client: Client<T>) {
            trace!("Client: Waiting for thread...");

            client.handle.unwrap().join().unwrap();
          }

          #[allow(unused)]
          pub fn close(self) {
            debug!("Client: Closing...");

            $crate::Network::<T>::close(&mut self.network.clone());

            Self::wait_thread(self);

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

          $(

            #[allow(unused)]
            pub fn $fn_name(&mut self, $($arg:$in_),*) -> $($out)* {
              let req_data = ($($arg,)*);
              let req_data_bytes = $crate::bincode::serialize(&req_data).unwrap();
              let req_bytes = $crate::prepend_u64($crate::hash_ident!($fn_name) as u64, req_data_bytes);
              let addr = self.get_serv_addr();

              debug!("Client: {} < {}", addr, stringify!($fn_name));

              let res = self.send(&addr, req_bytes);

              debug!("Client: {} > {}", addr, stringify!($fn_name));

              $crate::bincode::deserialize(&res).unwrap()
            }
          )*
        }

        // pub struct Duplex {
        //   network: $crate::Network<UdpTransport>,
        // }

        // impl Duplex {
        //   pub fn new(addr: &str) -> Duplex {
        //     Duplex{
        //       network: $crate::Network::new_default(&$crate::utils::to_socket_addr(addr)),
        //     }
        //   }

        //   pub fn listen(&self, addr: &str) -> Server<UdpTransport> {
        //     Service::listen_with_network(&mut self.network.clone())
        //   }

        //   pub fn connect(&self, addr: &str) -> Client<UdpTransport> {
        //     Service::connect_with_network(&mut self.network.clone(), addr.parse::<$crate::SocketAddr>().unwrap())
        //   }
        // }

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
            fn $fn_name( $( $arg : $in_ ),* ) $(-> $out)* $(| $error)* $block
          )*
        }

        #[allow(unused)]
        pub fn connect(addr: &str) -> Client<UdpTransport> {
          connect_with::<UdpTransport>(addr)
        }

        pub fn connect_with<T: 'static +  Transport>(addr: &str) -> Client<T> {
          let mut addr = to_socket_addr(addr.clone());

          let serv_addr = addr.clone();

          addr.set_port(0);

          let mut network = $crate::Network::new_default(&addr);

          connect_with_network(&mut network, serv_addr)
        }

        pub fn connect_with_network<T: 'static +  Transport>(network: &mut $crate::Network<T>, serv_addr: $crate::SocketAddr) -> Client<T> {
          info!("Client: Connecting {}", network.transport.get_addr());

          let network = network.clone();

          Client {
            serv_addr: serv_addr,
            network: network.clone(),
            handle: Some($crate::Network::async_read_loop(network.clone())),
          }
        }

        #[allow(unused)]
        pub fn listen(addr: &str) -> $crate::Server<$crate::UdpTransport> {
          listen_with::<UdpTransport>(addr)
        }


        #[allow(unused)]
        pub fn listen_with<T: 'static +  Transport>(addr: &str) -> $crate::Server<T> {

          let mut network = $crate::Network::new_default(&to_socket_addr(addr));

          listen_with_network(&mut network)
        }

        #[allow(unused)]
        pub fn listen_with_network<T: 'static +  Transport>(net: &mut $crate::Network<T>) -> $crate::Server<T> {
          info!("Server: Listening {}", net.transport.get_addr());

          let mut net = net.clone();

          let net_c = net.clone();

          let mut server = $crate::Server::new(net.clone());

          let interceptor = server.interceptor.clone();

          net.set_callback($crate::ServerCallback {
            closure: Box::new(move |pack| {
              let mut net = net_c.clone();

              let pack = interceptor.run(pack);

              let res = $service_name::dispatch(pack.clone());

              $crate::Network::send_answer(&mut net, &pack.header.sender, res, pack.header.msg_hash.clone());

              pack
            }),
          });

          server.handle = Some($crate::Network::async_read_loop(net.clone()));

          server
        }
      }

    )*
    // $(
      // pub use self::$service_name::*;
    // )*
  }
}

// // this macro expansion design took credits from tarpc by Google Inc.
// #[macro_export]
// macro_rules! service {
//   (
//     $(
//       $(#[$attr:meta])*
//       rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ) $(-> $out:ty)* $(| $error:ty)*;
//     )*
//   ) => {
//     service! {{
//       $(
//         $(#[$attr])*
//         rpc $fn_name( $( $arg : $in_ ),* ) $(-> $out)* $(| $error)*;
//       )*
//     }}
//   };
//   (
//     {
//       $(#[$attr:meta])*
//       rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ); // No return, no error

//       $( $unexpanded:tt )*
//     }
//     $( $expanded:tt )*
//   ) => {
//     service! {
//       { $( $unexpanded )* }

//       $( $expanded )*

//       $(#[$attr])*
//       rpc $fn_name( $( $arg : $in_ ),* ) -> () | ();
//     }
//   };
//   (
//     {
//       $(#[$attr:meta])*
//       rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ) -> $out:ty; //return, no error

//       $( $unexpanded:tt )*
//     }
//     $( $expanded:tt )*
//   ) => {
//     service! {
//       { $( $unexpanded )* }

//       $( $expanded )*

//       $(#[$attr])*
//       rpc $fn_name( $( $arg : $in_ ),* ) -> $out | ();
//     }
//   };
//   (
//     {
//       $(#[$attr:meta])*
//       rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ) | $error:ty; //no return, error

//       $( $unexpanded:tt )*
//     }
//     $( $expanded:tt )*
//   ) => {
//     service! {
//       { $( $unexpanded )* }

//       $( $expanded )*

//       $(#[$attr])*
//       rpc $fn_name( $( $arg : $in_ ),* ) -> () | $error;
//     }
//   };
//   (
//     {
//       $(#[$attr:meta])*
//       rpc $fn_name:ident( $( $arg:ident : $in_:ty ),* ) -> $out:ty | $error:ty; //return, error

//       $( $unexpanded:tt )*
//     }
//     $( $expanded:tt )*
//   ) => {
//     service! {
//       { $( $unexpanded )* }

//       $( $expanded )*

//       $(#[$attr])*
//       rpc $fn_name( $( $arg : $in_ ),* ) -> $out | $error;
//     }
//   };
//   (
//     {} // all expanded
//     $(
//       $(#[$attr:meta])*
//       rpc $fn_name:ident ( $( $arg:ident : $in_:ty ),* ) -> $out:ty | $error:ty;
//     )*
//   ) => {
//     pub use $crate::proto::Packet;
//     pub use $crate::server::{ Server, Interceptor };
//     pub use $crate::transport::{ Transport, UdpTransport };
//     pub use $crate::client::Client;
//     pub use $crate::utils::to_socket_addr;
//     pub use std::sync::{ Arc, Mutex };

//     // Main RPC service trait
//     pub trait RpcService {
//       $(
//         fn $fn_name($($arg:$in_),*) -> $out;
//       )*

//       fn get_fn_names() -> Arc<$crate::HashMap<usize, Callback>> {
//         let mut res = $crate::HashMap::new();

//         $(
//           let cb : Callback = Box::new(|pack: Packet| -> Vec<u8> {
//             let (_, body) = $crate::extract_u64_head(pack.data.clone()); // we can avoir extracting twice if we send the body to this closure

//             let ($($arg,)*) : ($($in_,)*) = $crate::bincode::deserialize(&body).unwrap();

//             debug!("Server: {} > {}", &pack.header.sender, stringify!($fn_name));

//             let call_res = &Self::$fn_name($($arg,)*);

//             debug!("Server: {} < {}", &pack.header.sender, stringify!($fn_name));

//             $crate::bincode::serialize(call_res).unwrap()
//           });

//           res.insert($crate::hash_ident!($fn_name), cb);
//         )*;

//         Arc::new(res)
//       }
//     }

//     impl ServiceTrait for RpcService {}

//     type Callback = Box<Fn(Packet) -> Vec<u8> + Send + Sync>;

//     lazy_static! {
//       pub static ref FN_IDS: Arc<$crate::HashMap<usize, Callback>> = RpcService::get_fn_names();
//     }



//     fn dispatch(pack: $crate::Packet) -> Vec<u8> {
//       let (func_id, _) = $crate::extract_u64_head(pack.data.clone());

//       if FN_IDS.len() == 0 {

//       }

//       let tocall = FN_IDS.get(&(func_id as usize)).unwrap();

//       tocall(pack)
//     }

//     pub trait ServiceTrait {
//       // Service methods
//       fn connect(addr: &str) -> Client<UdpTransport> {
//         Self::connect_with::<UdpTransport>(addr)
//       }

//       fn connect_with<T: 'static +  Transport>(addr: &str) -> Client<T> {
//         let mut addr = to_socket_addr(addr.clone());

//         let serv_addr = addr.clone();

//         addr.set_port(0);

//         let mut network = $crate::Network::new_default(&addr);

//         Self::connect_with_network(&mut network, serv_addr)
//       }

//       fn connect_with_network<T: 'static +  Transport>(network: &mut $crate::Network<T>, serv_addr: $crate::SocketAddr) -> Client<T> {
//         info!("Client: Connecting {}", network.transport.get_addr());

//         let mut network = network.clone();

//         Client {
//           serv_addr: serv_addr,
//           network: network.clone(),
//           handle: Some($crate::Network::async_read_loop(network.clone())),
//         }
//       }

//       fn listen(addr: &str) -> $crate::Server<$crate::UdpTransport> {
//         Self::listen_with::<UdpTransport>(addr)
//       }

//       fn listen_with<T: 'static +  Transport>(addr: &str) -> $crate::Server<T> {

//         let mut network = $crate::Network::new_default(&to_socket_addr(addr));

//         Self::listen_with_network(&mut network)
//       }

//       fn listen_with_network<T: 'static +  Transport>(net: &mut $crate::Network<T>) -> $crate::Server<T> {
//         info!("Server: Listening {}", net.transport.get_addr());

//         let mut net = net.clone();

//         let net_c = net.clone();

//         let mut server = $crate::Server::new(net.clone());

//         let interceptor = server.interceptor.clone();

//         net.set_callback($crate::ServerCallback {
//           closure: Box::new(move |pack| {
//             let mut net = net_c.clone();

//             let pack = interceptor.run(pack);

//             let res = dispatch(pack.clone());

//             $crate::Network::send_answer(&mut net, &pack.header.sender, res, pack.header.msg_hash.clone());

//             pack
//           }),
//         });

//         server.handle = Some($crate::Network::async_read_loop(net.clone()));

//         server
//       }
//     }

//     pub trait RpcClient {
//       fn get_serv_addr(&mut self) -> $crate::SocketAddr;
//       fn send(&mut self, addr: &$crate::SocketAddr, data: Vec<u8>) -> Vec<u8>;

//       $(
//         fn $fn_name(&mut self, $($arg:$in_),*) -> $out {
//           let req_data = ($($arg,)*);
//           let req_data_bytes = $crate::bincode::serialize(&req_data).unwrap();
//           let req_bytes = $crate::prepend_u64($crate::hash_ident!($fn_name) as u64, req_data_bytes);
//           let addr = self.get_serv_addr();

//           debug!("Client: {} < {}", addr, stringify!($fn_name));

//           let res = self.send(&addr, req_bytes);

//           debug!("Client: {} > {}", addr, stringify!($fn_name));

//           $crate::bincode::deserialize(&res).unwrap()
//         }
//       )*

//     }

//     pub struct Service;

//     impl ServiceTrait for Service {}

//     impl<T: 'static + Transport> RpcClient for Client<T> {
//       fn get_serv_addr(&mut self) -> $crate::SocketAddr {
//         self.serv_addr.clone()
//       }

//       fn send(&mut self, addr: &$crate::SocketAddr, data: Vec<u8>) -> Vec<u8> {
//         self.network.send(addr, data)
//       }
//     }

//     pub struct Duplex {
//       network: $crate::Network<UdpTransport>,
//     }

//     impl Duplex {
//       pub fn new(addr: &str) -> Duplex {
//         Duplex{
//           network: $crate::Network::new_default(&$crate::utils::to_socket_addr(addr)),
//         }
//       }

//       pub fn listen(&self, addr: &str) -> Server<UdpTransport> {
//         Service::listen_with_network(&mut self.network.clone())
//       }

//       pub fn connect(&self, addr: &str) -> Client<UdpTransport> {
//         Service::connect_with_network(&mut self.network.clone(), addr.parse::<$crate::SocketAddr>().unwrap())
//       }
//     }

//     pub trait RpcDuplex {
//       $(
//         fn $fn_name($($arg:$in_),*) -> $out;
//       )*

//       // fn new() -> Duplex {
//         // Duplex::new()
//       // }

//     }
//   }
// }
