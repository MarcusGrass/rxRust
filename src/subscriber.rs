use crate::prelude::*;
use std::rc::{Rc};
use std::cell::RefCell;
use std::sync::{Weak, Arc, Mutex, RwLock};
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::SinkExt;


/// Implements the Observer trait and Subscription trait. While the Observer is
/// the public API for consuming the values of an Observable, all Observers get
/// converted to a Subscriber, in order to provide Subscription capabilities.
///
pub trait Subscriber: Observer + SubscriptionLike {

  fn connect(&mut self, chn: SubscriptionChannel<Self::Item, Self::Err>);
}

pub fn pub_sub_channels<Item, Err>() -> (PublisherChannel<Item, Err>, SubscriptionChannel<Item, Err>){
  let item_channel = channel(usize::MAX);
  let err_channel = channel(1);
  let complete_channel = channel(1);
  let request_channel = channel(usize::MAX);
  let unsub_channel = channel(1);
  (PublisherChannel {
    item_chn: item_channel.0,
    err_chn: err_channel.0,
    complete_chn: complete_channel.0,
    request_chn: request_channel.1,
    unsub_chn: unsub_channel.1,
  }, SubscriptionChannel {
    item_chn: item_channel.1,
    err_chn: err_channel.1,
    complete_chn: complete_channel.1,
    request_chn: request_channel.0,
    unsub_chn: unsub_channel.0
  })
}

pub enum Upstream<Item, Err> {
  UNINIT,
  INIT(SubscriptionChannel<Item, Err>)
}

impl<Item, Err> SubscriptionLike for Upstream<Item, Err> {
  fn request(&mut self, requested: usize) {
    match self {
      Upstream::UNINIT => panic!("Request on uninitialized upstream"),
      Upstream::INIT(chn) => chn.request(requested)
    }
  }

  fn unsubscribe(&mut self) {
    match self {
      Upstream::UNINIT => panic!("Request on uninitialized upstream"),
      Upstream::INIT(chn) => chn.unsubscribe()
    }
  }

  fn is_closed(&self) -> bool {
    todo!()
  }
}



pub struct SubscriptionChannel<Item, Err> {
  pub(crate) item_chn: Receiver<Item>,
  pub(crate) err_chn: Receiver<Err>,
  pub(crate) complete_chn: Receiver<()>,
  pub(crate) request_chn: Sender<usize>,
  pub(crate) unsub_chn: Sender<()>,
}

pub struct PublisherChannel<Item, Err> {
  pub(crate) item_chn: Sender<Item>,
  pub(crate) err_chn: Sender<Err>,
  pub(crate) complete_chn: Sender<()>,
  pub(crate) request_chn: Receiver<usize>,
  pub(crate) unsub_chn: Receiver<()>,
}

impl<Item, Err> Observer for PublisherChannel<Item, Err> {
  type Item = Item;
  type Err = Err;

  fn next(&mut self, value: Self::Item) {
    let result = self.item_chn.try_send(value);
    if result.is_err() {
      println!("{:?}" ,result.err().unwrap());
      panic!()
    }
    result.ok().unwrap();
  }

  fn error(&mut self, err: Self::Err) {
    self.err_chn.try_send(err).unwrap();
  }

  fn complete(&mut self) {
    self.complete_chn.try_send(());
  }
}

impl<Item, Err> SubscriptionLike for SubscriptionChannel<Item, Err> {
  fn request(&mut self, requested: usize) {
    self.request_chn.try_send(requested).unwrap();
  }

  fn unsubscribe(&mut self) {
    self.unsub_chn.try_send(()).unwrap();
  }

  fn is_closed(&self) -> bool {
    todo!()
  }
}

pub trait LocalSubscriber<'a> : Observer + SubscriptionLike {
  fn on_subscribe(&self, sub: LocalSubscription<'a>);
}

pub trait SharedSubscriber: Observer + SubscriptionLike {

  fn on_subscribe(&self, sub: SharedSubscription);
}

impl<S: ?Sized> Subscriber for Box<S> where S: Subscriber {
  fn connect(&mut self, chn: SubscriptionChannel<Self::Item, Self::Err>) {
    (**self).connect(chn)
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
