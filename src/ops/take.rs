use crate::prelude::*;
use crate::{complete_proxy_impl, error_proxy_impl, is_stopped_proxy_impl};
use std::ops::Sub;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct TakeOp<S> {
  pub(crate) source: S,
  pub(crate) count: u32,
}

pub struct TakeOpSubscription<S> {
  pub(crate) source: Arc<RwLock<Option<S>>>,
  pub(crate) count: u128,
  pub(crate) done: bool,
}

impl<S> Clone for TakeOpSubscription<S> {
  fn clone(&self) -> Self {
    if self.source.read().unwrap().is_none() {
      TakeOpSubscription{
        source: self.source.clone(),
        count: self.count,
        done: self.done
      }
    } else {
      panic!()
    }
  }
}

impl<S> SubscriptionLike for TakeOpSubscription<S> where S: SubscriptionLike {
  fn request(&mut self, requested: u128) {
    if !self.done {
      self.source.write().unwrap().as_mut().unwrap().request(self.count); //TODO: Only request once and enable requesting less at a time
      self.done = true;
    }
  }

  fn unsubscribe(&mut self) {
    self.source.write().unwrap().as_mut().unwrap().unsubscribe();
  }

  fn is_closed(&self) -> bool {
    self.source.write().unwrap().as_mut().unwrap().is_closed() // Todo: check correctness
  }
}


observable_proxy_impl!(TakeOp, S);

impl<'a, S> LocalObservable<'a> for TakeOp<S>
where
  S: LocalObservable<'a>,
{
  type Unsub = LocalSubscription<'a>;
  fn actual_subscribe<O>(
    self,
    subscriber: Subscriber<O, LocalSubscription<'a>>,
  ) -> Self::Unsub
    where O: Observer<Item=Self::Item,Err= Self::Err> + 'a {
    let take_op_subscription = TakeOpSubscription { source: Arc::new(RwLock::new(None)), count: self.count as u128, done: false };
    let mut subscriber = Subscriber {
    observer: TakeObserver {
      source_sub: take_op_subscription.clone(),
      observer: subscriber.observer,
      count: self.count,
      hits: 0,
    },
      subscription: subscriber.subscription,
    };
    let source_sub = self.source.actual_subscribe(subscriber);
    *take_op_subscription.source.write().unwrap() = Some(source_sub);
    LocalSubscription::new(take_op_subscription)
    }
}

impl<S> SharedObservable for TakeOp<S>
where
  S: SharedObservable,
{
  type Unsub = SharedSubscription;
  fn actual_subscribe<O>(
    self,
    subscriber: Subscriber<O, SharedSubscription>,
  ) -> Self::Unsub
    where O: Observer<Item=Self::Item,Err= Self::Err> + Send + Sync + 'static {
    let take_op_subscription = TakeOpSubscription { source: Arc::new(RwLock::new(None)), count: self.count as u128, done: false };
    let subscriber = Subscriber {
      observer: TakeObserver {
        source_sub: take_op_subscription.clone(),
        observer: subscriber.observer,
        count: self.count,
        hits: 0,
      },
      subscription: subscriber.subscription,
    };
    let source_sub = self.source.actual_subscribe(subscriber);
    *take_op_subscription.source.write().unwrap() = Some(source_sub);
    SharedSubscription::new(take_op_subscription)
  }
}

pub struct TakeObserver<O, S> {
  source_sub: TakeOpSubscription<S>,
  observer: O,
  count: u32,
  hits: u32,
}

impl<O, S, Item, Err> Observer for TakeObserver<O, S>
where
  O: Observer<Item = Item, Err = Err>,
  S: SubscriptionLike
{
  type Item = Item;
  type Err = Err;
  fn next(&mut self, value: Item) {
    if self.hits < self.count {
      self.hits += 1;
      self.observer.next(value);
      if self.hits == self.count {
        self.complete();
        self.source_sub.source.write().unwrap().as_mut().unwrap().unsubscribe();
      }
    }
  }
  error_proxy_impl!(Err, observer);
  complete_proxy_impl!(observer);
  is_stopped_proxy_impl!(observer);
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn base_function() {
    let mut completed = false;
    let mut next_count = 0;

    observable::from_iter(0..100)
      .take(5)
      .subscribe_complete(|_| next_count += 1, || completed = true);

    assert_eq!(next_count, 5);
    assert!(completed);
  }

  #[test]
  fn take_support_fork() {
    let mut nc1 = 0;
    let mut nc2 = 0;
    {
      let take5 = observable::from_iter(0..100).take(5);
      let f1 = take5.clone();
      let f2 = take5;

      f1.take(5).subscribe(|_| nc1 += 1);
      f2.take(5).subscribe(|_| nc2 += 1);
    }
    assert_eq!(nc1, 5);
    assert_eq!(nc2, 5);
  }

  #[test]
  fn ininto_shared() {
    observable::from_iter(0..100)
      .take(5)
      .take(5)
      .into_shared()
      .subscribe(|_| {});
  }

  #[test]
  fn bench() { do_bench(); }

  benchmark_group!(do_bench, bench_take);

  fn bench_take(b: &mut bencher::Bencher) { b.iter(base_function); }
}
