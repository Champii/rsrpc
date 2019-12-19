use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use super::Transport;

pub struct UdpTransport {
    pub addr: SocketAddr,
    pub socket: Option<UdpSocket>,
    pub running: Arc<RwLock<bool>>,
    pub receiver: Arc<Mutex<Receiver<(Vec<u8>, SocketAddr)>>>,
    pub sender: Sender<(Vec<u8>, SocketAddr)>,
}

unsafe impl Send for UdpTransport {}
unsafe impl Sync for UdpTransport {}

impl UdpTransport {
    fn set_running(&mut self, running: bool) {
        let mut guard = self.running.write().unwrap();

        *guard = running;
    }

    // fn get_running(&mut self) -> bool {
    //     let guard = self.running.read().unwrap();

    //     (*guard).clone()
    // }

    fn socket_read(&self, addr: SocketAddr, stream: UdpSocket) {
        stream
            .set_read_timeout(Some(std::time::Duration::from_millis(10)))
            .unwrap();

        let stream = stream.try_clone().unwrap();

        let running = self.running.clone();
        let sender = self.sender.clone();

        thread::spawn(move || {
            let mut is_running = true;

            while is_running {
                is_running = *running.read().unwrap();

                let mut buff = [0; 100 * 1024];

                match stream.recv_from(&mut buff) {
                    Ok((amount, from)) => {
                        trace!("Read {} from {}", amount, from);

                        if amount == 0 && is_running {
                            trace!("Forced read 0 and not running");

                            break;
                        } else {
                            let res = buff[..amount].to_vec();

                            if buff.len() > 0 {
                                sender.send((res, addr)).unwrap();
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        if !*running.read().unwrap() {
                            drop(sender);

                            break;
                        }

                        continue;
                    }
                    Err(e) => {
                        error!("Error: {}", e);

                        drop(sender);

                        // drop(*sender.lock().unwrap());

                        break;
                    }
                };
            }
        });
    }
}

impl Clone for UdpTransport {
    fn clone(&self) -> Self {
        let socket = match self.socket.as_ref() {
            Some(s) => Some(s.try_clone().unwrap()),
            None => None,
        };

        UdpTransport {
            socket,
            addr: self.addr.clone(),
            running: self.running.clone(),
            receiver: self.receiver.clone(),
            sender: self.sender.clone(),
        }
    }
}

impl Transport for UdpTransport {
    fn new(addr: &SocketAddr) -> UdpTransport {
        let (sender, receiver) = channel();

        UdpTransport {
            addr: addr.clone(),
            socket: None,
            running: Arc::new(RwLock::new(false)),
            receiver: Arc::new(Mutex::new(receiver)),
            sender,
        }
    }

    fn listen(&mut self) {
        let socket = UdpSocket::bind(self.addr).unwrap();

        self.socket_read(self.addr, socket.try_clone().unwrap());

        self.socket = Some(socket);

        self.set_running(true);
    }

    fn connect(&mut self) -> Result<(), String> {
        let client_addr = self.addr.clone();

        self.addr = "127.0.0.1:0".parse().unwrap();

        self.listen();

        self.addr = client_addr;

        Ok(())
    }

    fn get_addr(&self) -> SocketAddr {
        // TODO: return the address from the socket instead
        // self.socket.as_ref().map(|socket | socket.local_addr().unwrap()).unwrap()

        self.addr
    }

    fn send(&mut self, addr: &SocketAddr, buff: Vec<u8>) -> bool {
        if let Some(s) = self.socket.as_ref() {
            if let Err(_) = s.send_to(buff.as_slice(), addr) {
                return false;
            }
        }

        trace!("Sent {} to {}", buff.len(), addr);

        true
    }

    fn get_recv(&mut self) -> Arc<Mutex<Receiver<(Vec<u8>, SocketAddr)>>> {
        self.receiver.clone()
    }

    // fn recv(&mut self) -> Result<(Vec<u8>, SocketAddr), Error> {
    //     let mut buff = [0; 100 * 1024];

    //     if !self.get_running() {
    //         return Err(Error::new(ErrorKind::Other, "Not running"));
    //     }

    //     match self.socket.as_ref().unwrap().recv_from(&mut buff) {
    //         Ok((amount, from)) => {
    //             trace!("Read {} from {}", amount, from);

    //             if amount == 0 && !self.get_running() {
    //                 trace!("Forced read 0 and not running");

    //                 Err(Error::new(ErrorKind::Other, "Read 0"))
    //             } else {
    //                 Ok((buff[..amount].to_vec(), from))
    //             }
    //         }
    //         Err(e) => Err(e),
    //     }
    // }

    fn is_running(&mut self) -> bool {
        self.running.read().unwrap().clone()
    }

    fn close(&mut self) {
        self.set_running(false);

        drop(self.socket.take().unwrap());
    }
}
