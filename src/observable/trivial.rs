use crate::prelude::*;
use crate::subscriber::Sub;

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

impl<'a, Err: Clone + 'a> LocalPublisherFactory<'a> for ThrowPublisherFactory<Err> {
  fn subscribe<S>(self, subscriber: S) where
      S: Sub<Item=Self::Item, Err=Self::Err> + 'a {
    {
      let mut publisher = ThrowPublisher {
        err: self.0,
        sub: subscriber,
      };

      publisher.sub.on_subscribe(&mut publisher);
    }
  }
}
/*

 */

impl<Err: Clone + Send + Sync + 'static> SharedPublisherFactory
  for ThrowPublisherFactory<Err>
{
  fn subscribe<S>(self, subscriber: S) where
      S: Sub<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
    let mut sub = ThrowPublisher {
      err: self.0,
      sub: subscriber,
    };
    sub.sub.on_subscribe(&mut sub);
  }
}

#[derive(Clone)]
struct ThrowPublisher<Err, S> {
  err: Err,
  sub: S,
}

impl<'a, Err: Clone, S> SubscriptionLike for ThrowPublisher<Err, S>
where
  S: Sub<Err=Err>
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

struct EmptyPublisher<S>(S);

impl<S> SubscriptionLike for EmptyPublisher<S>
where
  S: Sub
{
  fn request(&mut self, _: usize) { self.0.complete(); }

  fn unsubscribe(&mut self) {}

  fn is_closed(&self) -> bool { todo!() }
}

impl<'a, Item> LocalPublisherFactory<'a> for EmptyPublisherFactory<Item> {
  fn subscribe<S>(self, subscriber: S) -> LocalSubscription<'a> where
      S: Sub<Item=Self::Item, Err=Self::Err> + 'a {
    LocalSubscription::new(EmptyPublisher(subscriber))
  }
}


impl<Item> SharedPublisherFactory for EmptyPublisherFactory<Item> {
  fn subscribe<S>(self, subscriber: S) -> SharedSubscription where
      S: Sub<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
    SharedSubscription::new(EmptyPublisher(subscriber))
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
  fn subscribe<S>(self, subscriber: S) -> LocalSubscription<'a> where
      S: Sub<Item=Self::Item, Err=Self::Err> + 'a {
    LocalSubscription::new(subscriber)
  }
}

impl SharedPublisherFactory for NeverEmitterPublisherFactory {
  fn subscribe<S>(self, subscriber: S) -> SharedSubscription where
      S: Sub<Upstream=SharedSubscription, Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
    SharedSubscription::new(subscriber)
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
    let mut hits = 0;
    let mut completed = false;
    observable::empty().subscribe_complete(|()| hits += 1, || completed = true);

    assert_eq!(hits, 0);
    assert!(completed);
  }
}
