use crate::prelude::*;
use std::iter::{Repeat, Take};
use crate::subscriber::Subscriber;
use std::sync::Arc;

/// Creates an observable that produces values from an iterator.
///
/// Completes when all elements have been emitted. Never emits an error.
///
/// # Arguments
///
/// * `iter` - An iterator to get all the values from.
///
/// # Examples
///
/// A simple example for a range:
///
/// ```
/// use rxrust::prelude::*;
///
/// observable::from_iter(0..10)
///   .subscribe(|v| {println!("{},", v)});
/// ```
///
/// Or with a vector:
///
/// ```
/// use rxrust::prelude::*;
///
/// observable::from_iter(vec![0,1,2,3])
///   .subscribe(|v| {println!("{},", v)});
/// ```
pub fn from_iter<Iter, Item>(iter: Iter) -> ObservableBase<IterPublisherFactory<Iter>>
where
  Iter: IntoIterator<Item = Item>,
{
  ObservableBase::new(IterPublisherFactory(iter))
}

#[derive(Clone)]
pub struct IterPublisher<Iter, S> {
  it: Iter,
  sub: S,
  cursor: usize,
}

#[derive(Clone)]
pub struct IterPublisherFactory<Iter>(Iter);

impl<Iter> PublisherFactory for IterPublisherFactory<Iter>
where
  Iter: IntoIterator,
{
  type Item = Iter::Item;
  type Err = ();
}

impl<'a, Iter> LocalPublisherFactory<'a> for IterPublisherFactory<Iter>
where Iter: IntoIterator + Clone + 'a
{

  fn subscribe<S>(self, subscriber: S)  where
      S: Subscriber<Item=Self::Item, Err=Self::Err> + 'a {
    let mut publisher = Arc::new(IterPublisher {
      it: self.0,
      sub: subscriber,
      cursor: 0,
    });
    publisher.sub.on_subscribe(Arc::downgrade(&publisher.clone()))
  }
}

impl<Iter> SharedPublisherFactory for IterPublisherFactory<Iter>
where
  Iter: IntoIterator + Send + Sync + Clone + 'static,
{
  fn subscribe<S>(self, mut subscriber: S) where
      S: Subscriber<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
    let mut publisher = Arc::new(IterPublisher {
      it: self.0,
      sub: subscriber,
      cursor: 0,
    });
    publisher.sub.on_subscribe(Arc::downgrade(&publisher.clone()))
  }
}

impl<Iter, S> SubscriptionLike for IterPublisher<Iter, S>
where
  Iter: IntoIterator + Clone,
  S: Subscriber<Item = Iter::Item>,
{
  fn request(&mut self, requested: usize) {
    let mut it = self.it.clone().into_iter().skip(self.cursor as usize);
    let mut provided = 0;
    let mut v: Option<S::Item>;
    loop {
      v = it.next();
      if v.is_some() {
        self.sub.next(v.unwrap());
        provided += 1;
      } else {
        self.sub.complete();
        break;
      }
      if provided >= requested {
        break;
      }
    }
    self.cursor += provided;
  }

  fn unsubscribe(&mut self) { todo!() }

  fn is_closed(&self) -> bool { todo!() }
}


/// Creates an observable producing same value repeated N times.
///
/// Completes immediately after emitting N values. Never emits an error.
///
/// # Arguments
///
/// * `v` - A value to emits.
/// * `n` - A number of time to repeat it.
///
/// # Examples
///
/// ```
/// use rxrust::prelude::*;
///
/// observable::repeat(123, 3)
///   .subscribe(|v| {println!("{},", v)});
///
/// // print log:
/// // 123
/// // 123
/// // 123
/// ```
pub fn repeat<Item>(
  v: Item,
  n: usize,
) -> ObservableBase<IterPublisherFactory<Take<Repeat<Item>>>>
where
  Item: Clone,
{
  from_iter(std::iter::repeat(v).take(n))
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use bencher::Bencher;

  #[test]
  fn from_range() {
    let mut hit_count = 0;
    let mut completed = false;
    observable::from_iter(0..100)
      .subscribe_complete(|_| hit_count += 1, || completed = true);

    assert_eq!(hit_count, 100);
    assert!(completed);
  }

  #[test]
  fn from_vec() {
    let mut hit_count = 0;
    let mut completed = false;
    observable::from_iter(vec![0; 100])
      .subscribe_complete(|_| hit_count += 1, || completed = true);

    assert_eq!(hit_count, 100);
    assert!(completed);
  }

  #[test]
  fn repeat_three_times() {
    let mut hit_count = 0;
    let mut completed = false;
    repeat(123, 5).subscribe_complete(
      |v| {
        hit_count += 1;
        assert_eq!(123, v);
      },
      || completed = true,
    );
    assert_eq!(5, hit_count);
    assert!(completed);
  }

  #[test]
  fn repeat_zero_times() {
    let mut hit_count = 0;
    let mut completed = false;
    repeat(123, 0).subscribe_complete(
      |v| {
        hit_count += 1;
        assert_eq!(123, v);
      },
      || completed = true,
    );
    assert_eq!(0, hit_count);
    assert!(completed);
  }
  #[test]
  fn bench() { do_bench(); }

  benchmark_group!(do_bench, bench_from_iter);

  fn bench_from_iter(b: &mut Bencher) { b.iter(from_range); }
}
