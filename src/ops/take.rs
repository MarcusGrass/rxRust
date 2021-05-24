use crate::prelude::*;
use crate::{complete_proxy_impl, error_proxy_impl, is_stopped_proxy_impl};

#[derive(Clone)]
pub struct TakeOp<S> {
  pub(crate) source: S,
  pub(crate) count: u32,
}

#[derive(Clone)]
pub struct TakeOpSubscription<S> {
  pub(crate) source: S,
  pub(crate) count: u128,
  pub(crate) done: bool,
}

impl<S> SubscriptionLike for TakeOpSubscription<S> where S: SubscriptionLike {
  fn request(&mut self, requested: u128) {
    if !self.done {
      self.source.request(self.count); //TODO: Only request once and enable requesting less at a time
      self.done = true;
    }
  }

  fn unsubscribe(&mut self) {
    self.source.unsubscribe();
  }

  fn is_closed(&self) -> bool {
    self.source.is_closed() // Todo: check correctness
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
    let subscriber = Subscriber {
    observer: TakeObserver {
      observer: subscriber.observer,
      count: self.count,
      hits: 0,
    },
      subscription: subscriber.subscription,
    };
    let source_sub = self.source.actual_subscribe(subscriber);
    LocalSubscription::new(TakeOpSubscription { source: source_sub, count: self.count as u128, done: false })

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
    let subscriber = Subscriber {
      observer: TakeObserver {
        observer: subscriber.observer,
        count: self.count,
        hits: 0,
      },
      subscription: subscriber.subscription,
    };
    let source_sub = self.source.actual_subscribe(subscriber);
    SharedSubscription::new(TakeOpSubscription{
      source: source_sub,
      count: self.count as u128,
      done: false
    })
  }
}

pub struct TakeObserver<O> {
  observer: O,
  count: u32,
  hits: u32,
}

impl<O, Item, Err> Observer for TakeObserver<O>
where
  O: Observer<Item = Item, Err = Err>,
{
  type Item = Item;
  type Err = Err;
  fn next(&mut self, value: Item) {
    if self.hits < self.count {
      self.hits += 1;
      self.observer.next(value);
      if self.hits == self.count {
        self.complete();
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
