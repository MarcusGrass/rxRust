use crate::prelude::*;
use std::rc::{Rc};
use std::cell::RefCell;
use std::sync::{Weak, Arc, Mutex, RwLock};

/// Implements the Observer trait and Subscription trait. While the Observer is
/// the public API for consuming the values of an Observable, all Observers get
/// converted to a Subscriber, in order to provide Subscription capabilities.
///
pub trait Subscriber: Observer + SubscriptionLike {
  fn on_subscribe<S: SubscriptionLike>(&self, sub: Weak<S>);
}

impl<P> Subscriber for Rc<RefCell<P>> where P: Subscriber {

  fn on_subscribe<S: SubscriptionLike>(&self, sub: Weak<S>) {
    self.borrow().on_subscribe(sub)
  }
}


impl<S> SubscriptionLike for Arc<RwLock<S>> where S: SubscriptionLike {
  fn request(&mut self, requested: usize) {
    self.write().unwrap().request(requested)
  }

  fn unsubscribe(&mut self) {
    self.write().unwrap().unsubscribe();
    todo!()
  }

  fn is_closed(&self) -> bool {
    todo!()
  }
}
