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
    trace!("Add waiting {}", hash);

    self.waiting.insert(hash, tx);
  }

  pub fn resolve(matcher: &mut AsyncResponseMatcher, hash: String, data: Vec<u8>) {
    trace!("Resolve waiting {}", hash);

    match matcher.waiting.remove(&hash) {
      Some(tx) => tx.send(data).unwrap(),
      None => warn!("Cannot find such answer ! {}", hash),
    };
  }
}
