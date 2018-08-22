use std::sync::Arc;

use super::utils::Mutexed;

#[derive(Clone)]
pub struct Interceptor<T>(pub Arc<Mutexed<Arc<Fn(T) -> T>>>);

impl<T> Interceptor<T> {
  pub fn new() -> Interceptor<T> {
    Interceptor(Arc::new(Mutexed::new(Arc::new(|a| { a }))))
  }

  pub fn set(&mut self, cb: Arc<Fn(T) -> T>) {
    let mut guard = self.0.mutex.lock().unwrap();

    *guard = cb;
  }

  pub fn run(&self, t: T) -> T {
    (self.0.get())(t)
  }
}