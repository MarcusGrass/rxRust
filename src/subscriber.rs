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

pub fn pub_sub_channels<Item: Send + 'static, Err: Send + 'static>() -> (PublisherChannel<Item, Err>, SubscriptionChannel<Item, Err>){
  let item_channel = channel(1000);
  let request_channel = channel(1000);
  (PublisherChannel {
    item_chn: item_channel.0,
    request_chn: request_channel.1,
  }, SubscriptionChannel {
    item_chn: item_channel.1,
    request_chn: request_channel.0,
  })
}

pub enum Upstream<Item: Send + 'static, Err: Send + 'static> {
  UNINIT,
  INIT(SubscriptionChannel<Item, Err>)
}

pub enum SubscriberChannelItem<Item: Send + 'static, Err: Send + 'static> {
  Item(Item),
  Err(Err),
  Complete,
}

pub enum PublisherChannelItem {
  Request(usize),
  Cancel,
}



pub struct SubscriptionChannel<Item: Send + 'static, Err: Send + 'static> {
  pub(crate) item_chn: Receiver<SubscriberChannelItem<Item, Err>>,
  pub(crate) request_chn: Sender<PublisherChannelItem>,
}

pub struct PublisherChannel<Item: Send + 'static, Err: Send + 'static> {
  pub(crate) item_chn: Sender<SubscriberChannelItem<Item, Err>>,
  pub(crate) request_chn: Receiver<PublisherChannelItem>,
}

impl<Item: Send + 'static, Err: Send + 'static> Observer for Sender<SubscriberChannelItem<Item, Err>> {
  type Item = Item;
  type Err = Err;

  fn next(&mut self, value: Self::Item) {
    let result = self.try_send(SubscriberChannelItem::Item(value));
    if result.is_err() {
      println!("{:?}" ,result.err().unwrap());
      panic!()
    }
    result.ok().unwrap();
  }

  fn error(&mut self, err: Self::Err) {
    self.try_send(SubscriberChannelItem::Err(err)).unwrap();
  }

  fn complete(&mut self) {
    self.try_send(SubscriberChannelItem::Complete);
  }
}

impl SubscriptionLike for Sender<PublisherChannelItem> {
  fn request(&mut self, requested: usize) {
    self.try_send(PublisherChannelItem::Request(requested)).unwrap();
  }

  fn unsubscribe(&mut self) {
    self.try_send(PublisherChannelItem::Cancel).unwrap();
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
