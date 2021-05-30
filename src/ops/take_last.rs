use crate::prelude::*;
use crate::{error_proxy_impl};
use crate::subscriber::Subscriber;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct TakeLastOp<S> {
  pub(crate) source: S,
  pub(crate) count: usize,
}

#[doc(hidden)]
macro_rules! observable_impl {
  ($subscription:ty, $($marker:ident +)* $lf: lifetime) => {
  fn actual_subscribe<O>(
    self,
    subscriber: O,
  )
  where O: $subscription<Item=Self::Item,Err= Self::Err> + $($marker +)* $lf {
    /*
    let subscriber = Subscriber {
      observer: TakeLastObserver {
        observer: subscriber.observer,
        count: self.count,
        queue: VecDeque::new(),
      },
      subscription: subscriber.subscription,
    };
    self.source.actual_subscribe(subscriber)

     */
  }
}
}

observable_proxy_impl!(TakeLastOp, S);

impl<'a, S> LocalObservable<'a> for TakeLastOp<S>
where
  S: LocalObservable<'a> + 'a,
{
  fn actual_subscribe<Sub: Subscriber<LocalSubscription<'a>, Item=Self::Item, Err=Self::Err> + 'a>(self, subscriber: Sub) {
    todo!()
  }
}

impl<S> SharedObservable for TakeLastOp<S>
where
  S: SharedObservable,
  S::Item: Send + Sync + 'static,
{
  fn actual_subscribe<
    Sub: Subscriber<SharedSubscription, Item=Self::Item, Err=Self::Err> + Sync + Send + 'static
  >(self, subscriber: Sub) {
    todo!()
  }
}

pub struct TakeLastObserver<O, Item> {
  observer: O,
  count: usize,
  queue: VecDeque<Item>, // TODO: replace VecDeque with RingBuf
}

impl<Item, Err, O> Observer for TakeLastObserver<O, Item>
where
  O: Observer<Item = Item, Err = Err>,
{
  type Item = Item;
  type Err = Err;
  fn next(&mut self, value: Item) {
    self.queue.push_back(value);
    while self.queue.len() > self.count {
      self.queue.pop_front();
    }
  }
  error_proxy_impl!(Err, observer);
  fn complete(&mut self) {
    for value in self.queue.drain(..) {
      self.observer.next(value);
    }
    self.observer.complete();
  }

}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn base_function() {
    let mut completed = false;
    let mut ticks = vec![];

    observable::from_iter(0..100)
      .take_last(5)
      .subscribe_complete(|v| ticks.push(v), || completed = true);

    assert_eq!(ticks, vec![95, 96, 97, 98, 99]);
    assert!(completed);
  }

  #[test]
  fn take_last_support_fork() {
    let mut nc1 = 0;
    let mut nc2 = 0;
    {
      let take_last5 = observable::from_iter(0..100).take_last(5);
      let f1 = take_last5.clone();
      let f2 = take_last5;

      f1.take_last(5).subscribe(|_| nc1 += 1);
      f2.take_last(5).subscribe(|_| nc2 += 1);
    }
    assert_eq!(nc1, 5);
    assert_eq!(nc2, 5);
  }

  #[test]
  fn ininto_shared() {
    observable::from_iter(0..100)
      .take_last(5)
      .take_last(5)
      .into_shared()
      .subscribe(|_| {});
  }

  #[test]
  fn bench() { do_bench(); }

  benchmark_group!(do_bench, bench_take_last);

  fn bench_take_last(b: &mut bencher::Bencher) { b.iter(base_function); }
}
