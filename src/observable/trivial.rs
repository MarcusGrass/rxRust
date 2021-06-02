use crate::prelude::*;
use crate::subscriber::Subscriber;
use std::sync::Arc;
use std::rc::Rc;

/// Creates an observable that emits no items, just terminates with an error.
///
/// # Arguments
///
/// * `e` - An error to emit and terminate with
pub fn throw<Err>(e: Err) -> ObservableBase<ThrowPublisherFactory<Err>> {
  ObservableBase::new(ThrowPublisherFactory(e))
}

#[derive(Clone)]
pub struct ThrowPublisherFactory<Err>(Err);

impl<Err> PublisherFactory for ThrowPublisherFactory<Err> {
  type Item = ();
  type Err = Err;
}

impl<'a, Err: Clone + Send + 'static> LocalPublisherFactory<'a> for ThrowPublisherFactory<Err> {
  fn subscribe<S>(self, subscriber: S) where
      S: Subscriber<Item=Self::Item, Err=Self::Err> + Send + 'static {
    {
      let (p, s) = pub_sub_channels();
      let mut publisher = ThrowPublisher {
        err: self.0,
        sub: p,
      };
      start_publish_loop(publisher, subscriber)
    }
  }
}

impl<Err: Clone + Send + Sync + 'static> SharedPublisherFactory
  for ThrowPublisherFactory<Err>
{
  fn subscribe<S>(self, mut subscriber: S) where
      S: Subscriber<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
    let (p, s) = pub_sub_channels();
    let mut publisher = ThrowPublisher {
      err: self.0,
      sub: p,
    };
    start_publish_loop(publisher, subscriber)
  }
}

struct ThrowPublisher<Err: Clone> {
  err: Err,
  sub: PublisherChannel<(), Err>,
}

impl<Err: Clone + Send + 'static> Source for ThrowPublisher<Err>
{
  type Item = ();
  type Err = Err;

  fn get_channel(&self) -> &PublisherChannel<Self::Item, Self::Err> {
    &self.sub
  }
}

impl<Err: Clone> SubscriptionLike for ThrowPublisher<Err>
where
{
  fn request(&mut self, _: usize) { self.sub.error(self.err.clone()); }

  fn unsubscribe(&mut self) {}

  fn is_closed(&self) -> bool { todo!() }
}

/// Creates an observable that produces no values.
///
/// Completes immediately. Never emits an error.
///
/// # Examples
/// ```
/// use rxrust::prelude::*;
///
/// observable::empty()
///   .subscribe(|v: &i32| {println!("{},", v)});
///
/// // Result: no thing printed
/// ```
pub fn empty<Item>() -> ObservableBase<EmptyPublisherFactory<Item>> {
  ObservableBase::new(EmptyPublisherFactory(TypeHint::new()))
}

#[derive(Clone)]
pub struct EmptyEmitter<Item>(TypeHint<Item>);

#[derive(Clone)]
pub struct EmptyPublisherFactory<Item>(TypeHint<Item>);

impl<Item> PublisherFactory for EmptyPublisherFactory<Item> {
  type Item = Item;
  type Err = ();
}

struct EmptyPublisher<Item>(PublisherChannel<Item, ()>);
impl<Item: Send + 'static> Source for EmptyPublisher<Item> {
  type Item = Item;
  type Err = ();

  fn get_channel(&self) -> &PublisherChannel<Self::Item, Self::Err> {
    &self.0
  }
}

impl<Item> SubscriptionLike for EmptyPublisher<Item>
{
  fn request(&mut self, _: usize) { self.0.complete(); }

  fn unsubscribe(&mut self) {}

  fn is_closed(&self) -> bool { todo!() }
}

impl<'a, Item: Send + 'static> LocalPublisherFactory<'a> for EmptyPublisherFactory<Item> {
  fn subscribe<S>(self, mut subscriber: S) where
      S: Subscriber<Item=Self::Item, Err=Self::Err> + Send + 'static {
    let (p, s) = pub_sub_channels();
    let mut publisher = EmptyPublisher(p);
    subscriber.connect(s);
    start_publish_loop(publisher, subscriber)
  }
}


impl<Item: Send + 'static> SharedPublisherFactory for EmptyPublisherFactory<Item> {
  fn subscribe<S>(self, mut subscriber: S) where
      S: Subscriber<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
    let (p, s) = pub_sub_channels();
    let mut publisher = EmptyPublisher(p);
    subscriber.connect(s);
    start_publish_loop(publisher, subscriber)
  }
}
/// Creates an observable that never emits anything.
///
/// Neither emits a value, nor completes, nor emits an error.
pub fn never() -> ObservableBase<NeverEmitterPublisherFactory> {
  ObservableBase::new(NeverEmitterPublisherFactory)
}

#[derive(Clone)]
pub struct NeverEmitter();

#[derive(Clone)]
pub struct NeverEmitterPublisherFactory;

impl PublisherFactory for NeverEmitterPublisherFactory {
  type Item = ();
  type Err = ();
}

impl<'a> LocalPublisherFactory<'a> for NeverEmitterPublisherFactory {
  fn subscribe<S>(self, _subscriber: S) where
      S: Subscriber<Item=Self::Item, Err=Self::Err> + 'a {
  }
}

impl SharedPublisherFactory for NeverEmitterPublisherFactory {
  fn subscribe<S>(self, _subscriber: S) where
      S: Subscriber<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn throw() {
    let mut value_emitted = false;
    let mut completed = false;
    let mut error_emitted = String::new();
    observable::throw(String::from("error")).subscribe_all(
      // helping with type inference
      |_| value_emitted = true,
      |e| error_emitted = e,
      || completed = true,
    );
    assert!(!value_emitted);
    assert!(!completed);
    assert_eq!(error_emitted, "error");
  }

  #[test]
  fn empty() {
    /*
    let mut hits = 0;
    let mut completed = false;
    observable::empty().subscribe_complete(|()| hits += 1, || completed = true);

    assert_eq!(hits, 0);
    assert!(completed);

     */
  }
}
