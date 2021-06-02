#![cfg(test)]
use crate::prelude::*;
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::time::Duration;

#[derive(Clone)]
pub struct ObserverBlockAll<N, E, C, Sub, Item, Err> {
  next: N,
  error: E,
  complete: C,
  upstream: Arc<RwLock<Sub>>,
  is_stopped: Arc<Mutex<bool>>,
  marker: TypeHint<(*const Item, *const Err)>,
}

impl<Item, Err, Sub, N, E, C> Subscriber for ObserverBlockAll<N, E, C, Sub, Item, Err>
  where
      C: FnMut(),
      N: FnMut(Item),
      E: FnMut(Err),
      Sub: SubscriptionLike,
{

  fn connect(&mut self, chn: SubscriptionChannel<Self::Item, Self::Err>) {
  }
}

impl<Item, Err, S, N, E, C> SubscriptionLike for ObserverBlockAll<N, E, C, S, Item, Err>
  where
      C: FnMut(),
      N: FnMut(Item),
      E: FnMut(Err),
      S: SubscriptionLike,
{

  fn request(&mut self, _requested: usize) {
  }

  fn unsubscribe(&mut self) {
    self.upstream.write().unwrap().unsubscribe();
  }

  fn is_closed(&self) -> bool {
      todo!()
  }
}
impl<Item, Err, S, N, E, C> Observer for ObserverBlockAll<N, E, C, S, Item, Err>
where
  C: FnMut(),
  N: FnMut(Item),
  E: FnMut(Err),
  S: SubscriptionLike,
{
  type Item = Item;
  type Err = Err;
  #[inline(always)]
  fn next(&mut self, value: Self::Item) { (self.next)(value); }

  fn error(&mut self, err: Self::Err) {
    (self.error)(err);
    *self.is_stopped.lock().unwrap() = true;
  }

  fn complete(&mut self) {
    (self.complete)();
    *self.is_stopped.lock().unwrap() = true;
  }

}

pub trait SubscribeBlockingAll<'a, N, E, C> {
  /// A type implementing [`SubscriptionLike`]
  type Unsub: SubscriptionLike;

  /// Invokes an execution of an Observable that will block the subscribing
  /// thread; useful for testing and last resort blocking in token scenarios.
  ///
  /// Will return a SubscriptionWrapper only after upstream completion.
  ///
  /// Should preferably not be used in production because of both its blocking
  /// nature, as well as its implementation by an arbitrarily chosen 1ms
  /// thread sleep which goes against reactive programming philosophy.
  ///
  /// Use with caution, will block forever if the upstream never completes or
  /// errors out.
  ///
  /// * `error`: A handler for a terminal event resulting from an error.
  /// * `complete`: A handler for a terminal event resulting from successful
  /// completion.
  fn subscribe_blocking_all(
    self,
    next: N,
    error: E,
    complete: C,
  ) -> SubscriptionWrapper<Self::Unsub>;
}

impl<'a, S, N, E, C> SubscribeBlockingAll<'a, N, E, C> for Shared<S>
where
  S: SharedObservable,
  N: FnMut(S::Item) + Send + Sync + 'static,
  E: FnMut(S::Err) + Send + Sync + 'static,
  C: FnMut() + Send + Sync + 'static,
  S::Err: 'static,
  S::Item: 'static,
{
  type Unsub = SharedSubscription;

  fn subscribe_blocking_all(
    self,
    next: N,
    error: E,
    complete: C,
  ) -> SubscriptionWrapper<Self::Unsub>
  where
    Self: Sized,
  {
    let stopped = Arc::new(Mutex::new(false));
    let stopped_c = Arc::clone(&stopped);
    let subscriber = ObserverBlockAll {
      next,
      error,
      complete,
      upstream: Arc::new(RwLock::new(SharedSubscription::default())),
      is_stopped: stopped,
      marker: TypeHint::new(),
    };
    self.actual_subscribe(subscriber);
    while !*stopped_c.lock().unwrap() {
      std::thread::sleep(Duration::from_millis(1))
    }
    SubscriptionWrapper(SharedSubscription::default())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use futures::executor::ThreadPool;
  use std::sync::{Arc, Mutex};
  use std::time::{Duration, Instant};

  #[test]
  fn blocks_shared() {
    let pool = ThreadPool::new().unwrap();
    let stamp = Instant::now();
    let interval = observable::interval(Duration::from_millis(1), pool)
      .take(5)
      .into_shared();

    let first = Arc::new(Mutex::new(0));
    let first_clone = Arc::clone(&first);
    interval.clone().subscribe_blocking_all(
      move |_| *first_clone.lock().unwrap() += 1,
      |_| {},
      || {},
    );
    assert_eq!(*first.lock().unwrap(), 5);

    let second = Arc::new(Mutex::new(0));
    let second_clone = Arc::clone(&second);
    interval.subscribe_blocking_all(
      move |_| *second_clone.lock().unwrap() += 1,
      |_| {},
      || {},
    );

    assert_eq!(*first.lock().unwrap(), 5);
    assert_eq!(*second.lock().unwrap(), 5);
    assert!(stamp.elapsed() > Duration::from_millis(10));
  }
}
