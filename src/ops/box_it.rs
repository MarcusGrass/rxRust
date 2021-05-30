use crate::prelude::*;

pub trait BoxObservable<'a> {
  type Item;
  type Err;
  fn box_subscribe(
    self: Box<Self>,
    subscriber: Box<dyn Subscriber<LocalSubscription<'a>, Item = Self::Item, Err = Self::Err> + 'a>,
  );
}

pub trait SharedBoxObservable {
  type Item;
  type Err;
  fn box_subscribe(
    self: Box<Self>,
    subscriber: Box<dyn Subscriber<SharedSubscription, Item = Self::Item, Err = Self::Err> + Send + Sync>,
  );
}

#[doc(hidden)]
macro_rules! box_observable_impl {
    ($subscription:ty, $source:ident, $($marker:ident +)* $lf: lifetime) => {
  type Item = $source::Item;
  type Err = $source::Err;
  fn box_subscribe(
    self: Box<Self>,
    subscriber: Box<dyn $subscription<$subscription, Item=Self::Item,Err= Self::Err> + $($marker +)* $lf>,
  ) {
    self.actual_subscribe(subscriber)
  }
}
}

impl<'a, T> BoxObservable<'a> for T
where
  T: LocalObservable<'a> + 'a,
{
  type Item = T::Item;
  type Err = T::Err;
  fn box_subscribe(
    self: Box<Self>,
    subscriber: Box<dyn Subscriber<LocalSubscription<'a>, Item = Self::Item, Err = Self::Err> + 'a>,
  ) {
    self.actual_subscribe(subscriber)
  }
}

impl<T> SharedBoxObservable for T
where
  T: SharedObservable,
  T::Item: Send + Sync + 'static,
  T::Err: Send + Sync + 'static,
{
  type Item = T::Item;
  type Err = T::Err;

  fn box_subscribe(self: Box<Self>, subscriber: Box<dyn Subscriber<SharedSubscription, Item=Self::Item, Err=Self::Err> + Send + Sync>) {
    self.actual_subscribe(subscriber);
  }
}

pub struct BoxOp<T>(T);

impl<T: Clone> Clone for BoxOp<T> {
  #[inline]
  fn clone(&self) -> Self { BoxOp(self.0.clone()) }
}

pub type LocalBoxOp<'a, Item, Err> =
  BoxOp<Box<dyn BoxObservable<'a, Item = Item, Err = Err> + 'a>>;
pub type LocalCloneBoxOp<'a, Item, Err> =
  BoxOp<Box<dyn BoxClone<'a, Item = Item, Err = Err> + 'a>>;
pub type SharedBoxOp<Item, Err> =
  BoxOp<Box<dyn SharedBoxObservable<Item = Item, Err = Err> + Send + Sync>>;
pub type SharedCloneBoxOp<Item, Err> =
  BoxOp<Box<dyn SharedBoxClone<Item = Item, Err = Err>>>;

impl<'a, Item, Err> Observable for LocalBoxOp<'a, Item, Err> {
  type Item = Item;
  type Err = Err;
}
impl<'a, Item, Err> LocalObservable<'a> for LocalBoxOp<'a, Item, Err> {
  fn actual_subscribe<Sub: Subscriber<LocalSubscription<'a>, Item=Self::Item, Err=Self::Err> + 'a>(self, subscriber: Sub) {
    self.0.box_subscribe(Box::new(subscriber));
  }
}

impl<Item, Err> Observable for SharedBoxOp<Item, Err> {
  type Item = Item;
  type Err = Err;
}

impl<Item, Err> SharedObservable for SharedBoxOp<Item, Err> {
  fn actual_subscribe<
    S: Subscriber<SharedSubscription, Item=Self::Item, Err=Self::Err> + Sync + Send + 'static
  >(self, subscriber: S) {
    self.0.box_subscribe(Box::new(subscriber));
  }
}

impl<'a, Item, Err> Observable for LocalCloneBoxOp<'a, Item, Err> {
  type Item = Item;
  type Err = Err;
}
impl<'a, Item, Err> LocalObservable<'a> for LocalCloneBoxOp<'a, Item, Err> {
  fn actual_subscribe<Sub: Subscriber<LocalSubscription<'a>, Item=Self::Item, Err=Self::Err> + 'a>(self, subscriber: Sub) {
    self.0.box_subscribe(Box::new(subscriber));
  }
}

impl<Item, Err> Observable for SharedCloneBoxOp<Item, Err> {
  type Item = Item;
  type Err = Err;
}
impl<Item, Err> SharedObservable for SharedCloneBoxOp<Item, Err> {
  fn actual_subscribe<
    S: Subscriber<SharedSubscription, Item=Self::Item, Err=Self::Err> + Sync + Send + 'static
  >(self, subscriber: S) {
    self.0.box_subscribe(Box::new(subscriber));
  }
}

/// FIXME: IntoBox should use associated type instead of generic after rust
/// generic specialization supported and work for associated type. So we have
/// different specialized version for same type, and type infer will work fine.
pub trait IntoBox<T> {
  fn box_it(origin: T) -> BoxOp<Self>
  where
    Self: Sized;
}

impl<'a, T> IntoBox<T> for Box<dyn BoxObservable<'a, Item = T::Item, Err = T::Err> + 'a>
where
  T: LocalObservable<'a> + 'a,
{
  fn box_it(origin: T) -> BoxOp<Self> { BoxOp(Box::new(origin)) }
}

impl<T> IntoBox<T>
  for Box<dyn SharedBoxObservable<Item = T::Item, Err = T::Err> + Send + Sync>
where
  T: SharedObservable + Send + Sync + 'static,
  T::Item: Send + Sync + 'static,
  T::Err: Send + Sync + 'static,
{
  fn box_it(origin: T) -> BoxOp<Self> { BoxOp(Box::new(origin)) }
}

// support box observable clone
pub trait BoxClone<'a>: BoxObservable<'a> {
  fn box_clone(&self) -> Box<dyn BoxClone<'a, Item = Self::Item, Err = Self::Err> + 'a>;
}

impl<'a, T> BoxClone<'a> for T
where
  T: BoxObservable<'a> + Clone + 'a,
{
  fn box_clone(&self) -> Box<dyn BoxClone<'a, Item = Self::Item, Err = Self::Err> + 'a> {
    Box::new(self.clone())
  }
}

impl<'a, Item, Err> Clone for Box<dyn BoxClone<'a, Item = Item, Err = Err> + 'a> {
  #[inline]
  fn clone(&self) -> Self { self.box_clone() }
}

impl<'a, T> IntoBox<T> for Box<dyn BoxClone<'a, Item = T::Item, Err = T::Err> + 'a>
where
  T: LocalObservable<'a> + Clone + 'a,
{
  fn box_it(origin: T) -> BoxOp<Self> { BoxOp(Box::new(origin)) }
}

pub trait SharedBoxClone: SharedBoxObservable {
  fn box_clone(&self) -> Box<dyn SharedBoxClone<Item = Self::Item, Err = Self::Err>>;
}

impl<T> SharedBoxClone for T
where
  T: SharedBoxObservable + Clone + 'static,
{
  fn box_clone(&self) -> Box<dyn SharedBoxClone<Item = Self::Item, Err = Self::Err>> {
    Box::new(self.clone())
  }
}

impl<Item, Err> Clone for Box<dyn SharedBoxClone<Item = Item, Err = Err>> {
  #[inline]
  fn clone(&self) -> Self { self.box_clone() }
}

impl<'a, T> IntoBox<T> for Box<dyn SharedBoxClone<Item = T::Item, Err = T::Err>>
where
  T: SharedBoxObservable + Clone + 'static,
{
  fn box_it(origin: T) -> BoxOp<Self> { BoxOp(Box::new(origin)) }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use bencher::Bencher;
  use ops::box_it::{BoxClone, SharedBoxClone};
  use ops::box_it::{LocalBoxOp, SharedBoxOp};

  #[test]
  fn box_observable() {
    let mut test = 0;
    let mut boxed: LocalBoxOp<'_, i32, ()> = observable::of(100).box_it();
    boxed.subscribe(|v| test = v);

    boxed = observable::empty().box_it();
    boxed.subscribe(|_| unreachable!());
    assert_eq!(test, 100);
  }

  #[test]
  fn shared_box_observable() {
    let mut boxed: SharedBoxOp<i32, ()> = observable::of(100).box_it();
    boxed.into_shared().subscribe(|_| {});

    boxed = observable::empty().box_it();
    boxed.into_shared().subscribe(|_| unreachable!());
  }

  #[test]
  fn box_clone() {
    observable::of(100)
      .box_it::<Box<dyn BoxClone<Item = _, Err = _>>>()
      .clone()
      .subscribe(|_| {});
  }

  #[test]
  fn shared_box_clone() {
    observable::of(100)
      .box_it::<Box<dyn SharedBoxClone<Item = _, Err = _>>>()
      .clone()
      .into_shared()
      .subscribe(|_| {});
  }

  #[test]
  fn bench() { do_bench(); }

  benchmark_group!(do_bench, bench_box_clone);

  fn bench_box_clone(b: &mut Bencher) { b.iter(box_clone); }
}
