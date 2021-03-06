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
    pub use $crate::{ Transport, UdpTransport };

    $(
      #[allow(non_snake_case)]
      pub mod $service_name {
        pub use $crate::{ Transport, UdpTransport, TcpTransport };
        use $crate::utils::{to_socket_addr};
        use std::sync::{ Arc, Mutex };
        use std::net::SocketAddr;
        use lazy_static::*;
        use $crate::plugins::Wrapper;

        #[allow(unused)]
        #[derive(Clone)]
        pub struct $service_name {
          pub actual_sender: SocketAddr,
          $(pub $var: $type_,)*
        }

        impl $service_name {
          pub fn new() -> $service_name {
            $service_name {
              actual_sender: SocketAddr::new("127.0.0.1".parse().unwrap(), 0),
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

                ctx_c.actual_sender = pack.header.sender;

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
            trace!("Client: Closing...");

            self.network.close();

            self.wait();

            debug!("Client: Closed");
          }
          #[allow(unused)]
          fn get_serv_addr(&mut self) -> $crate::SocketAddr {
            self.serv_addr.clone()
          }

          #[allow(unused)]
          fn send(&mut self, addr: &$crate::SocketAddr, data: Vec<u8>) -> Result<Vec<u8>, String> {
            self.network.send(addr, data)
          }

          $(

            #[allow(unused)]
            pub fn $fn_name(&mut self, $($arg:$in_),*) -> Result<Result<$out, $error>, String> {
              let req_data = ($($arg,)*);
              let req_data_bytes = $crate::bincode::serialize(&req_data).unwrap();
              let req_bytes = $crate::prepend_u64($crate::hash_ident!($fn_name) as u64, req_data_bytes);
              let addr = self.get_serv_addr();

              debug!("Client: {} < {}", addr, stringify!($fn_name));

              let res = self.send(&addr, req_bytes);

              res.map(|data| {
                debug!("Client: {} > {}", addr, stringify!($fn_name));

                Ok($crate::bincode::deserialize(&data).unwrap())
              }).map_err(|err| {
                error!("Error client send for {}", stringify!($fn_name));

                Default::default()
              })

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
            trace!("Server: Closing...");

            self.network.close();

            self.wait();

            debug!("Server: Closed");
          }
        }

        #[allow(unused)]
        pub struct Duplex {
        }

        lazy_static! {
          pub static ref DUPLEX: Arc<Mutex<Option<$crate::Network<UdpTransport>>>> = Arc::new(Mutex::new(None));
        }

        impl Duplex {
          #[allow(unused)]
          pub fn listen(addr: &str) -> Server<UdpTransport> {
            let mut network = $crate::Network::new_default(&$crate::utils::to_socket_addr(addr));

            network.listen();

            let mut guard = DUPLEX.lock().unwrap();

            *guard = Some(network.clone());

            listen_with_network(network)
          }

          #[allow(unused)]
          pub fn connect(addr: &str) -> Client<UdpTransport> {
            let mut net = DUPLEX.lock().unwrap().as_ref().unwrap().clone();

            net.connect().unwrap();


            connect_with_network(net)
          }

          #[allow(unused)]
          pub fn wait() {
            trace!("Server: Waiting for thread...");

            let mut net;

            {
              let mut guard = DUPLEX.lock().unwrap();
              let mut n = (*guard).take().unwrap();

              net = n.clone();

              n.handle = None;

              *guard = Some(n);
            }

            net.wait();
          }

          #[allow(unused)]
          pub fn close() {
            trace!("Server: Closing...");

            let mut net;
            {
              let mut guard = DUPLEX.lock().unwrap();
              net = (*guard).take().unwrap().clone();
            }

            net.close();

            trace!("Server: Waiting for thread...");

            net.wait();

            debug!("Server: Closed");
          }

          #[allow(dead_code)]
          pub fn add_plugin<T: Wrapper + Clone + 'static>(plugin: T) {
            let mut guard = DUPLEX.lock().unwrap();

            for net in  (*guard).iter_mut() {
              net.plugins.add(plugin.clone());
            }
          }
        }

        impl ServiceTrait for $service_name {
          $(
            fn $fn_name(&mut $self_, $( $arg : $in_ ),* ) -> $out $block
          )*
        }

        #[allow(unused)]
        pub fn connect_udp(serv_addr: &str) -> Result<Client<UdpTransport>, String> {
          connect_with::<UdpTransport>(serv_addr)
        }

        #[allow(unused)]
        pub fn connect_tcp(serv_addr: &str) -> Result<Client<TcpTransport>, String> {
          connect_with::<TcpTransport>(serv_addr)
        }

        pub fn connect_with<T: 'static +  Transport>(serv_addr: &str) -> Result<Client<T>, String> {
          let mut network = $crate::Network::new_default(&to_socket_addr(serv_addr));

          if let Err(e) = network.connect() {
            return Err(e);
          }

          Ok(connect_with_network(network))
        }

        pub fn connect_with_network<T: 'static +  Transport>(network: $crate::Network<T>) -> Client<T> {
          debug!("Client: Connected {}", network.transport.get_addr());

          Client {
            serv_addr: network.transport.get_addr(),
            network,
          }
        }

        #[allow(unused)]
        pub fn listen_udp(addr: &str) -> Server<$crate::UdpTransport> {
          listen_with::<UdpTransport>(addr)
        }

        #[allow(unused)]
        pub fn listen_tcp(addr: &str) -> Server<$crate::TcpTransport> {
          listen_with::<TcpTransport>(addr)
        }

        #[allow(unused)]
        pub fn listen_with<T: 'static +  Transport>(addr: &str) -> Server<T> {
          let mut network = $crate::Network::new_default(&to_socket_addr(addr));

          network.listen();

          listen_with_network(network)
        }

        #[allow(unused)]
        pub fn listen_with_network<T: 'static +  Transport>(net: $crate::Network<T>) -> Server<T> {
          debug!("Server: Listening {}", net.transport.get_addr());
          let mut net_c = net.clone();

          net_c.handle = None;

          let mut server = Server::new(net_c.clone());

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
