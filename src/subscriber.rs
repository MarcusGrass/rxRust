use crate::prelude::*;
use std::rc::{Rc};
use std::cell::RefCell;
use std::sync::{Weak, Arc, Mutex, RwLock};

/// Implements the Observer trait and Subscription trait. While the Observer is
/// the public API for consuming the values of an Observable, all Observers get
/// converted to a Subscriber, in order to provide Subscription capabilities.
///
pub trait Subscriber<S>: Observer + SubscriptionLike {

  fn on_subscribe(&self, sub: S);
}

pub trait LocalSubscriber<'a> : Observer + SubscriptionLike {
  fn on_subscribe(&self, sub: LocalSubscription<'a>);
}

pub trait SharedSubscriber: Observer + SubscriptionLike {

  fn on_subscribe(&self, sub: SharedSubscription);
}

impl<S: ?Sized, T> Subscriber<T> for Box<S> where S: Subscriber<T> {
  fn on_subscribe(&self, sub: T) {
    (**self).on_subscribe(sub)
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

impl<S> SubscriptionLike for Rc<S> where S: SubscriptionLike {
  fn request(&mut self, requested: usize) {
    self.request(requested);
  }

  fn unsubscribe(&mut self) {
    self.unsubscribe();
  }

  fn is_closed(&self) -> bool {
    self.is_closed()
  }
}
impl<S> SubscriptionLike for Arc<S> where S: SubscriptionLike {
  fn request(&mut self, requested: usize) {
    self.request(requested);
  }

  fn unsubscribe(&mut self) {
    self.unsubscribe();
  }

  fn is_closed(&self) -> bool {
    self.is_closed()
  }
}
