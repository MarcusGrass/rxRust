use crate::prelude::*;
use std::sync::{Arc, Mutex};
use crate::subscriber::Subscriber;

type SharedPublishers<Item, Err> =
  Arc<Mutex<Vec<Box<dyn Publisher<Item = Item, Err = Err> + Send + Sync>>>>;

pub type SharedSubject<Item, Err> =
  Subject<SharedPublishers<Item, Err>, SharedSubscription>;

impl<Item, Err> SharedSubject<Item, Err> {
  #[inline]
  pub fn new() -> Self
  where
    Self: Default,
  {
    Self::default()
  }
  #[inline]
  pub fn subscribed_size(&self) -> usize {
    self.observers.observers.lock().unwrap().len()
  }
}

impl<Item: Send + 'static, Err: Send + 'static> Observable for SharedSubject<Item, Err> {
  type Item = Item;
  type Err = Err;
}

impl<Item: Send + 'static, Err: Send + 'static> SharedObservable for SharedSubject<Item, Err> {
  fn actual_subscribe(
    self,
    channel: PublisherChannel<Self::Item, Self::Err>,
  ) {
    /*
    let subscription = subscriber.subscription.clone();
    self.subscription.add(subscription.clone());
    self
      .observers
      .observers
      .lock()
      .unwrap()
      .push(Box::new(subscriber));

     */
  }
}

#[test]
fn smoke() {
  let test_code = Arc::new(Mutex::new("".to_owned()));
  let mut subject = SharedSubject::new();
  let c_test_code = test_code.clone();
  subject.clone().into_shared().subscribe(move |v: &str| {
    *c_test_code.lock().unwrap() = v.to_owned();
  });
  subject.next("test shared subject");
  assert_eq!(*test_code.lock().unwrap(), "test shared subject");
  assert_eq!(subject.subscribed_size(), 1);
}
