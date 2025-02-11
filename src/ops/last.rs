use crate::observer::error_proxy_impl;
use crate::prelude::*;

#[derive(Clone)]
pub struct LastOp<S, Item> {
  pub(crate) source: S,
  pub(crate) last: Option<Item>,
}

impl<Item, S> Observable for LastOp<S, Item>
where
  S: Observable<Item = Item>,
{
  type Item = Item;
  type Err = S::Err;
}

#[doc(hidden)]
macro observable_impl($subscription:ty, $($marker:ident +)* $lf: lifetime) {
  fn actual_subscribe<O>(
    self,
    subscriber: Subscriber<O, $subscription>,
  ) -> Self::Unsub
  where O: Observer<Item=Self::Item,Err= Self::Err> + $($marker +)* $lf {
    let subscriber = Subscriber {
      observer: LastObserver {
        observer: subscriber.observer,
        last: self.last,
      },
      subscription: subscriber.subscription,
    };
    self.source.actual_subscribe(subscriber)
  }
}

impl<'a, Item, S> LocalObservable<'a> for LastOp<S, Item>
where
  S: LocalObservable<'a, Item = Item>,
  Item: 'a + Clone,
{
  type Unsub = S::Unsub;
  observable_impl!(LocalSubscription, 'a);
}

impl<Item, S> SharedObservable for LastOp<S, Item>
where
  S: SharedObservable<Item = Item>,
  Item: Send + Sync + 'static + Clone,
{
  type Unsub = S::Unsub;
  observable_impl!(SharedSubscription, Send + Sync + 'static);
}

pub struct LastObserver<S, T> {
  observer: S,
  last: Option<T>,
}

impl<O, Item, Err> Observer for LastObserver<O, Item>
where
  O: Observer<Item = Item, Err = Err>,
  Item: Clone,
{
  type Item = Item;
  type Err = Err;
  fn next(&mut self, value: Item) { self.last = Some(value); }
  error_proxy_impl!(Err, observer);
  fn complete(&mut self) {
    if let Some(v) = &self.last {
      self.observer.next(v.clone())
    }
    self.observer.complete();
  }

  #[inline]
  fn is_stopped(&self) -> bool { self.observer.is_stopped() }
}

#[cfg(test)]
mod test {
  extern crate test;
  use crate::prelude::*;
  use test::Bencher;

  #[test]
  fn last_or_hundered_items() {
    let mut completed = 0;
    let mut errors = 0;
    let mut last_item = None;

    observable::from_iter(0..100).last_or(200).subscribe_all(
      |v| last_item = Some(v),
      |_| errors += 1,
      || completed += 1,
    );

    assert_eq!(errors, 0);
    assert_eq!(completed, 1);
    assert_eq!(Some(99), last_item);
  }

  #[test]
  fn last_or_no_items() {
    let mut completed = 0;
    let mut errors = 0;
    let mut last_item = None;

    observable::empty().last_or(100).subscribe_all(
      |v| last_item = Some(v),
      |_| errors += 1,
      || completed += 1,
    );

    assert_eq!(errors, 0);
    assert_eq!(completed, 1);
    assert_eq!(Some(100), last_item);
  }

  #[test]
  fn last_one_item() {
    let mut completed = 0;
    let mut errors = 0;
    let mut last_item = None;

    observable::from_iter(0..2).last().subscribe_all(
      |v| last_item = Some(v),
      |_| errors += 1,
      || completed += 1,
    );

    assert_eq!(errors, 0);
    assert_eq!(completed, 1);
    assert_eq!(Some(1), last_item);
  }

  #[test]
  fn last_no_items() {
    let mut completed = 0;
    let mut errors = 0;
    let mut last_item = None;

    observable::empty().last().subscribe_all(
      |v: i32| last_item = Some(v),
      |_| errors += 1,
      || completed += 1,
    );

    assert_eq!(errors, 0);
    assert_eq!(completed, 1);
    assert_eq!(None, last_item);
  }

  #[test]
  fn last_support_fork() {
    let mut value = 0;
    let mut value2 = 0;
    {
      let o = observable::from_iter(1..100).last();
      let o1 = o.clone().last();
      let o2 = o.last();
      o1.subscribe(|v| value = v);
      o2.subscribe(|v| value2 = v);
    }
    assert_eq!(value, 99);
    assert_eq!(value2, 99);
  }

  #[test]
  fn last_or_support_fork() {
    let mut default = 0;
    let mut default2 = 0;
    let o = observable::create(|mut subscriber| {
      subscriber.complete();
    })
    .last_or(100);
    let o1 = o.clone().last_or(0);
    let o2 = o.clone().last_or(0);
    o1.subscribe(|v| default = v);
    o2.subscribe(|v| default2 = v);
    assert_eq!(default, 100);
    assert_eq!(default, 100);
  }

  #[test]
  fn last_fork_and_shared() {
    observable::of(0)
      .last_or(0)
      .into_shared()
      .into_shared()
      .subscribe(|_| {});

    observable::of(0)
      .last()
      .into_shared()
      .into_shared()
      .subscribe(|_| {});
  }
  #[bench]
  fn bench_last(b: &mut Bencher) { b.iter(last_or_hundered_items); }
}
