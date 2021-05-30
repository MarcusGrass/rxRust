use crate::prelude::*;
use crate::{complete_proxy_impl, error_proxy_impl};
use crate::subscriber::Subscriber;

#[derive(Clone)]
pub struct FilterOp<S, F> {
  pub(crate) source: S,
  pub(crate) filter: F,
}

#[doc(hidden)]
macro_rules! observable_impl {
    ($subscription:ty, $source:ident, $($marker:ident +)* $lf: lifetime) => {
  fn actual_subscribe<O>(
    self,
    subscriber: O,
  )
  where O: $subscription<Item=Self::Item,Err= Self::Err> + $($marker +)* $lf {
    /*
    let filter = self.filter;
    self.source.actual_subscribe(Subscriber {
      observer: FilterObserver {
        filter,
        observer: subscriber.observer,
      },
      subscription: subscriber.subscription,
    });

     */
  }
}
}

impl<S, F> Observable for FilterOp<S, F>
where
  S: Observable,
  F: FnMut(&S::Item) -> bool,
{
  type Item = S::Item;
  type Err = S::Err;
}

impl<'a, S, F> LocalObservable<'a> for FilterOp<S, F>
where
  S: LocalObservable<'a>,
  F: FnMut(&S::Item) -> bool + 'a,
{
  fn actual_subscribe<Sub: Subscriber<LocalSubscription<'a>, Item=Self::Item, Err=Self::Err> + 'a>(self, subscriber: Sub) {
    todo!()
  }
}

impl<S, F> SharedObservable for FilterOp<S, F>
where
  S: SharedObservable,
  F: FnMut(&S::Item) -> bool + Send + Sync + 'static,
{
  fn actual_subscribe<
    Sub: Subscriber<SharedSubscription, Item=Self::Item, Err=Self::Err> + Sync + Send + 'static
  >(self, subscriber: Sub) {
    todo!()
  }
}

pub struct FilterObserver<S, F> {
  observer: S,
  filter: F,
}

impl<Item, Err, O, F> Observer for FilterObserver<O, F>
where
  O: Observer<Item = Item, Err = Err>,
  F: FnMut(&Item) -> bool,
{
  type Item = Item;
  type Err = Err;
  fn next(&mut self, value: Item) {
    if (self.filter)(&value) {
      self.observer.next(value)
    }
  }
  error_proxy_impl!(Err, observer);
  complete_proxy_impl!(observer);

}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn fork_and_shared() {
    observable::from_iter(0..10)
      .filter(|v| v % 2 == 0)
      .clone()
      .filter(|_| true)
      .clone()
      .into_shared()
      .subscribe(|_| {});
  }

  #[test]
  fn smoke() {
    observable::from_iter(0..1000)
      .filter(|v| v % 2 == 0)
      .subscribe(|v| {
        assert!(v % 2 == 0);
      });
  }

  #[test]
  fn bench() { do_bench(); }

  benchmark_group!(do_bench, bench_filter);

  fn bench_filter(b: &mut bencher::Bencher) { b.iter(smoke); }
}
