use crate::prelude::*;
use crate::subscriber::Subscriber;

/// param `subscribe`: the function that is called when the Observable is
/// initially subscribed to. This function is given a Subscriber, to which
/// new values can be `next`ed, or an `error` method can be called to raise
/// an error, or `complete` can be called to notify of a successful
/// completion.
pub fn create<F, S, Item, Err>(
  subscribe: F,
) -> ObservableBase<FnPublisherFactory<F, Item, Err>>
where
  F: FnOnce(Box<dyn Subscriber<S, Item = Item, Err = Err>>),
{
  ObservableBase::new(FnPublisherFactory(subscribe, TypeHint::new()))
}

#[derive(Clone)]
pub struct FnPublisherFactory<F, Item, Err>(F, TypeHint<(Item, Err)>);

impl<F, Item, Err> PublisherFactory for FnPublisherFactory<F, Item, Err> {
  type Item = Item;
  type Err = Err;
}

impl<'a, F: 'a, Item: 'a, Err: 'a> LocalPublisherFactory<'a>
  for FnPublisherFactory<F, Item, Err>
where
  F: FnOnce(Box<dyn Subscriber<LocalSubscription<'a>, Item = Item, Err = Err> + 'a>),
{

  fn subscribe<S>(self, subscriber: S) where
      S: Subscriber<LocalSubscription<'a>, Item=Self::Item, Err=Self::Err> + 'a {
    todo!()
  }
}

#[derive(Clone, PartialEq)]
pub enum CallFunc<F> {
  Noop,
  Call(F),
}

#[derive(Clone)]
pub struct LocalFnPublisher<F> {
  call: CallFunc<F>,
}

impl<F> SubscriptionLike for LocalFnPublisher<F>
where
  F: FnOnce() -> (),
{
  fn request(&mut self, _: usize) {
    let n: CallFunc<F> = CallFunc::Noop;

    match std::mem::replace(&mut self.call, n) {
      CallFunc::Noop => {}
      CallFunc::Call(f) => (f)(),
    }
  }

  fn unsubscribe(&mut self) {}

  fn is_closed(&self) -> bool {
    match self.call {
      CallFunc::Noop => true,
      CallFunc::Call(_) => false,
    }
  }
}

#[derive(Clone)]
struct SharedFnPublisher<F> {
  call: CallFunc<F>,
}

impl<F> SubscriptionLike for SharedFnPublisher<F>
where
  F: FnOnce() -> (),
{
  fn request(&mut self, _: usize) {
    let n: CallFunc<F> = CallFunc::Noop;

    match std::mem::replace(&mut self.call, n) {
      CallFunc::Noop => {}
      CallFunc::Call(f) => (f)(),
    }
  }

  fn unsubscribe(&mut self) {}

  fn is_closed(&self) -> bool {
    match self.call {
      CallFunc::Noop => true,
      CallFunc::Call(_) => false,
    }
  }
}

impl<F: 'static, Item: 'static, Err: 'static> SharedPublisherFactory
  for FnPublisherFactory<F, Item, Err>
where
    F: FnOnce(Box<dyn Subscriber<SharedSubscription, Item = Item, Err = Err> + 'static>) + Send + Sync,
{
  fn subscribe<S>(self, subscriber: S) where
      S: Subscriber<SharedSubscription, Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
    todo!()
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use bencher::Bencher;
  use std::sync::{Arc, Mutex};

  #[test]
  fn proxy_call() {
    let next = Arc::new(Mutex::new(0));
    let err = Arc::new(Mutex::new(0));
    let complete = Arc::new(Mutex::new(0));
    let c_next = next.clone();
    let c_err = err.clone();
    let c_complete = complete.clone();

    observable::create(|mut subscriber| {
      subscriber.next(&1);
      subscriber.next(&2);
      subscriber.next(&3);
      subscriber.complete();
      subscriber.next(&3);
      subscriber.error("never dispatch error");
    })
    .into_shared()
    .subscribe_all(
      move |_| *next.lock().unwrap() += 1,
      move |_: &str| *err.lock().unwrap() += 1,
      move || *complete.lock().unwrap() += 1,
    );

    assert_eq!(*c_next.lock().unwrap(), 3);
    assert_eq!(*c_complete.lock().unwrap(), 1);
    assert_eq!(*c_err.lock().unwrap(), 0);
  }
  #[test]
  fn support_fork() {
    let o = observable::create(|mut subscriber| {
      subscriber.next(&1);
      subscriber.next(&2);
      subscriber.next(&3);
      subscriber.next(&4);
    });
    let sum1 = Arc::new(Mutex::new(0));
    let sum2 = Arc::new(Mutex::new(0));
    let c_sum1 = sum1.clone();
    let c_sum2 = sum2.clone();
    o.clone().subscribe(move |v| *sum1.lock().unwrap() += v);
    o.clone().subscribe(move |v| *sum2.lock().unwrap() += v);

    assert_eq!(*c_sum1.lock().unwrap(), 10);
    assert_eq!(*c_sum2.lock().unwrap(), 10);
  }

  #[test]
  fn fork_and_share() {
    let observable = observable::create(|_| {});
    observable.clone().into_shared().subscribe(|_: i32| {});
    observable.clone().into_shared().subscribe(|_| {});

    let observable = observable::create(|_| {}).into_shared();
    observable.clone().subscribe(|_: i32| {});
    observable.clone().subscribe(|_| {});
  }

  #[test]
  fn bench() { do_bench(); }

  benchmark_group!(do_bench, bench_from_fn);

  fn bench_from_fn(b: &mut Bencher) { b.iter(proxy_call); }
}
