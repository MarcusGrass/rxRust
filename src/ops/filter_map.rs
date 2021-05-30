use crate::prelude::*;
use crate::{complete_proxy_impl, error_proxy_impl};
use crate::subscriber::Subscriber;

#[derive(Clone)]
pub struct FilterMapOp<S, F> {
  pub(crate) source: S,
  pub(crate) f: F,
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
    self.source.actual_subscribe(Subscriber {
      observer: FilterMapObserver {
        down_observer: subscriber.observer,
        f: self.f,
        _marker: TypeHint::new(),
      },
      subscription: subscriber.subscription,
    });

     */
  }
}
}

impl<'a, Item, S, F> Observable for FilterMapOp<S, F>
where
  S: Observable,
  F: FnMut(S::Item) -> Option<Item>,
{
  type Item = Item;
  type Err = S::Err;
}

impl<'a, Item, S, F> LocalObservable<'a> for FilterMapOp<S, F>
where
  S: LocalObservable<'a>,
  F: FnMut(S::Item) -> Option<Item> + 'a,
  S::Item: 'a,
{
  fn actual_subscribe<Sub: Subscriber<LocalSubscription<'a>, Item=Self::Item, Err=Self::Err> + 'a>(self, subscriber: Sub) {
    todo!()
  }
}

impl<Item, S, F> SharedObservable for FilterMapOp<S, F>
where
  S: SharedObservable,
  F: FnMut(S::Item) -> Option<Item> + Send + Sync + 'static,
  S::Item: 'static,
{
  fn actual_subscribe<
    Sub: Subscriber<SharedSubscription, Item=Self::Item, Err=Self::Err> + Sync + Send + 'static
  >(self, subscriber: Sub) {
    todo!()
  }
}

pub struct FilterMapObserver<O, F, Item> {
  down_observer: O,
  f: F,
  _marker: TypeHint<*const Item>,
}

impl<O, F, Item, Err, OutputItem> Observer for FilterMapObserver<O, F, Item>
where
  O: Observer<Item = OutputItem, Err = Err>,
  F: FnMut(Item) -> Option<OutputItem>,
{
  type Item = Item;
  type Err = Err;
  fn next(&mut self, value: Item) {
    if let Some(v) = (self.f)(value) {
      self.down_observer.next(v)
    }
  }
  error_proxy_impl!(Err, down_observer);
  complete_proxy_impl!(down_observer);

}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn map_types_mixed() {
    let mut i = 0;
    observable::from_iter(vec!['a', 'b', 'c'])
      .filter_map(|_v| Some(1))
      .subscribe(|v| i += v);
    assert_eq!(i, 3);
  }

  #[test]
  fn filter_map_shared_and_fork() {
    observable::of(1)
      .filter_map(|_| Some("str"))
      .clone()
      .into_shared()
      .subscribe(|_| {});
  }

  #[test]
  fn filter_map_return_ref() {
    observable::of(&1)
      .filter_map(Some)
      .clone()
      .into_shared()
      .subscribe(|_| {});
  }

  #[test]
  fn bench() { do_bench(); }

  benchmark_group!(do_bench, bench_map_types_mixed);

  fn bench_map_types_mixed(b: &mut bencher::Bencher) { b.iter(map_types_mixed); }
}
