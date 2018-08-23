use std::sync::{ Arc, Mutex };
use super::proto::Packet;
use std::fmt;

pub trait Wrapper: fmt::Debug + Send + Sync {
  fn on_send(&self, pack: &Packet) -> Packet;
  fn on_recv(&self, pack: &Packet) -> Packet;
}

lazy_static! {
  pub static ref PLUGINS: super::Mutexed<Plugins> = super::Mutexed::new(super::Plugins::new());
}

#[derive(Clone)]
pub struct Plugins {
  wrappers: Arc<Mutex<Vec<Box<Wrapper>>>>,
}

impl Plugins {
  pub fn new() -> Plugins {
    Plugins {
      wrappers: Default::default(),
    }
  }

  pub fn add<T: Wrapper + 'static>(wrapper: T) {
    let mut plugs = PLUGINS.get();

    info!("({}) Adding plugin {:?}", plugs.wrappers.lock().unwrap().len() + 1, wrapper);

    plugs.add_(wrapper);
  }

  fn add_<T: Wrapper + 'static>(&mut self, wrapper: T) {

    let mut guard = self.wrappers.lock().unwrap();

    (*guard).push(Box::new(wrapper));
  }

  pub fn run_on_send(&mut self, data: Packet) -> Packet {
    trace!("Processing Plugins on Send request");

    let mut data = data.clone();

    let guard = self.wrappers.lock().unwrap();

    for wrapper in (*guard).iter() {
      trace!("- {:?}", wrapper);

      data = wrapper.on_send(&data);
    }

    data
  }

  pub fn run_on_recv(&mut self, data: Packet) -> Packet {
    trace!("Processing Plugins on Recv request");

    let mut data = data.clone();

    let guard = self.wrappers.lock().unwrap();

    for wrapper in (*guard).iter().rev() {
      trace!("- {:?}", wrapper);

      data = wrapper.on_recv(&data);
    }

    data
  }
}

