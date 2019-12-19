use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use super::Transport;

pub struct TcpTransport {
    pub addr: SocketAddr,
    pub serv_addr: SocketAddr,
    pub server: Option<TcpListener>,
    pub clients: Arc<RwLock<HashMap<String, TcpStream>>>, // socket_addr =>Client
    pub running: Arc<RwLock<bool>>,
    pub receiver: Arc<Mutex<Receiver<(Vec<u8>, SocketAddr)>>>,
    pub sender: Arc<Mutex<Sender<(Vec<u8>, SocketAddr)>>>,
}

unsafe impl Send for TcpTransport {}
unsafe impl Sync for TcpTransport {}

impl TcpTransport {
    fn set_running(&mut self, running: bool) {
        let mut guard = self.running.write().unwrap();

        *guard = running;
    }

    fn get_running(&mut self) -> bool {
        let guard = self.running.read().unwrap();

        (*guard).clone()
    }

    fn socket_read(&self, addr: SocketAddr, stream: TcpStream) {
        stream
            .set_read_timeout(Some(std::time::Duration::from_millis(10)))
            .unwrap();

        let mut stream = stream.try_clone().unwrap();

        let running = self.running.clone();
        let sender = self.sender.clone();

        thread::spawn(move || {
            while running.read().unwrap().clone() {
                let mut buff = vec![];

                // is_running = running.read().unwrap().clone();

                match stream.read_to_end(&mut buff) {
                    Ok(_) => {
                        if !*running.read().unwrap() {
                            break;
                        }

                        if buff.len() > 0 {
                            sender.lock().unwrap().send((buff, addr)).unwrap();
                        } else {
                            break;
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        if !*running.read().unwrap() {
                            break;
                        }

                        if buff.len() > 0 {
                            sender.lock().unwrap().send((buff, addr)).unwrap();
                        }
                    }
                    Err(e) => {
                        if e.kind() != ErrorKind::Other {
                            error!("Error read {}", e);
                        }

                        break;
                    }
                }
            }
        });
    }
}

impl Clone for TcpTransport {
    fn clone(&self) -> Self {
        let server = match self.server.as_ref() {
            Some(s) => Some(s.try_clone().unwrap()),
            None => None,
        };

        // let clients = HashMap::new();

        // for (addr, client) in *self.clients.read().unwrap() {
        //     clients.insert(addr.clone(), client.try_clone().unwrap());
        // }

        TcpTransport {
            server,
            clients: self.clients.clone(),
            addr: self.addr.clone(),
            serv_addr: self.addr.clone(),
            running: self.running.clone(),
            receiver: self.receiver.clone(),
            sender: self.sender.clone(),
        }
    }
}

impl Transport for TcpTransport {
    fn new(addr: &SocketAddr) -> TcpTransport {
        let (sender, receiver) = channel();

        TcpTransport {
            addr: addr.clone(),
            serv_addr: addr.clone(),
            server: None,
            clients: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            receiver: Arc::new(Mutex::new(receiver)),
            sender: Arc::new(Mutex::new(sender)),
        }
    }

    fn listen(&mut self) {
        let socket = TcpListener::bind(self.addr).unwrap();

        let clients = self.clients.clone();

        self.server = Some(socket.try_clone().unwrap());

        self.set_running(true);

        let mut local_self = self.clone();

        thread::spawn(move || {
            while local_self.get_running() {
                match socket.accept() {
                    Ok((stream, addr)) => {
                        debug!("Accept {:?}", addr.clone());

                        local_self.socket_read(addr, stream.try_clone().unwrap());

                        clients.write().unwrap().insert(addr.to_string(), stream);
                    }
                    Err(e) => {
                        error!("Error accept {}", e);

                        break;
                    }
                }
            }
        });
    }

    fn connect(&mut self) -> Result<(), String> {
        let socket = TcpStream::connect(self.addr);

        if socket.is_err() {
            return Err(socket.unwrap_err().to_string());
        }

        let socket = socket.unwrap();

        self.clients
            .write()
            .unwrap()
            .insert(self.addr.to_string(), socket.try_clone().unwrap());

        let addr = self.addr.clone();

        self.set_running(true);

        self.socket_read(addr, socket);

        Ok(())
    }

    fn get_addr(&self) -> SocketAddr {
        // TODO: return the address from the socket instead
        // self.socket.as_ref().map(|socket | socket.local_addr().unwrap()).unwrap()

        self.addr

        // if self.server.is_some() {
        //     self.addr
        // } else {
        //     let mut c = self.addr;

        //     for (_, client) in self.clients.read().unwrap().iter() {
        //         c = client.local_addr().unwrap();
        //     }

        //     c
        // }
    }

    fn send(&mut self, addr: &SocketAddr, buff: Vec<u8>) -> bool {
        let mut clients = self.clients.write().unwrap();

        if let Some(s) = clients.get_mut(&addr.to_string()) {
            if let Err(_) = s.write_all(buff.as_slice()) {
                return false;
            }

            trace!("Sent {} to {}", buff.len(), addr);
        }

        true
    }

    // fn recv(&mut self) -> Result<(Vec<u8>, SocketAddr), Error> {
    //     if !*self.running.read().unwrap() {
    //         Err(std::io::Error::new(
    //             std::io::ErrorKind::Interrupted,
    //             "Closed",
    //         ))
    //     } else if self.req_buffer.read().unwrap().len() == 0 {
    //         Err(std::io::Error::new(
    //             std::io::ErrorKind::WouldBlock,
    //             "WouldBlock",
    //         ))
    //     } else {
    //         Ok(self.req_buffer.write().unwrap().remove(0))
    //     }
    // }
    fn get_recv(&mut self) -> Arc<Mutex<Receiver<(Vec<u8>, SocketAddr)>>> {
        self.receiver.clone()
    }

    fn is_running(&mut self) -> bool {
        self.running.read().unwrap().clone()
    }

    fn close(&mut self) {
        self.set_running(false);

        if self.server.is_some() {
            drop(self.server.take());
        }
        // drop(self.clients);
    }
}
