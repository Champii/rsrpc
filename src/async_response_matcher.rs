use std::collections::HashMap;
use futures::channel::oneshot;

pub struct AsyncResponseMatcher {
  waiting: HashMap<String, oneshot::Sender<Vec<u8>>>,
}

impl AsyncResponseMatcher {
  pub fn new() -> Self {
    Self {
      waiting: HashMap::new(),
    }
  }

  pub fn add(&mut self, hash: String, tx: oneshot::Sender<Vec<u8>>) {
    self.waiting.insert(hash, tx);
  }

  pub fn resolve(matcher: &mut AsyncResponseMatcher, hash: String, data: Vec<u8>) {
    match matcher.waiting.remove(&hash) {
      Some(tx) => tx.send(data).unwrap(),
      None => (),
    };
  }
}
