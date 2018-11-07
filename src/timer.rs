use std::thread;
use std::time::Duration;
use super::oneshot::{ channel, Receiver };

pub struct Timer {

}

impl Timer {
  pub fn new<T: 'static +  Send + Sync>(wait_time: Duration, err: T) -> Receiver<T> {
    let (tx, rx) = channel::<T>();

    thread::spawn(move || {
      thread::sleep(wait_time);

      tx.send(err);
    });

    rx
  }
}
