use super::oneshot::{channel, Receiver};
use std::fmt::Debug;
use std::thread;
use std::time::Duration;

pub struct Timer {}

impl Timer {
  pub fn new<T: 'static + Send + Sync + Debug>(wait_time: Duration, err: T) -> Receiver<T> {
    let (tx, rx) = channel::<T>();

    thread::spawn(move || {
      thread::sleep(wait_time);

      tx.send(err).unwrap();
    });

    rx
  }
}
