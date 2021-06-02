use crate::prelude::*;
use std::iter::{Repeat, Take};
use crate::subscriber::Subscriber;
use std::sync::Arc;
use std::rc::Rc;
use std::sync::mpsc::channel;

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
pub fn from_iter<Iter, Item>(iter: Iter) -> ObservableBase<IterPublisherFactory<Iter, Item>>
where
  Iter: IntoIterator<Item = Item>,
{
  ObservableBase::new(IterPublisherFactory(iter, TypeHint::new()))
}

pub struct IterPublisher<Iter, Item>
  where
      Iter: IntoIterator<Item=Item>,
{
  it: Iter,
  cursor: usize,
  observer: PublisherChannel<Iter::Item, ()>,
}

#[derive(Clone)]
pub struct IterPublisherFactory<Iter, Item>(Iter, TypeHint<Item>);

impl<Iter, Item> PublisherFactory for IterPublisherFactory<Iter, Item>
where
  Item: Send + 'static,
  Iter: IntoIterator<Item=Item>,
{
  type Item = Iter::Item;
  type Err = ();
}

impl<'a, Iter, Item> LocalPublisherFactory<'a> for IterPublisherFactory<Iter, Item>
where
    Item: Send + 'static,
    Iter: IntoIterator<Item=Item> + Clone + Send + Sync + 'static
{

  fn subscribe<S>(self, subscriber: S)  where
      S: Subscriber<Item=Self::Item, Err=Self::Err> + Send + 'static {
    let (pub_ch, sub_ch) = pub_sub_channels();
    let mut publisher = IterPublisher {
      it: self.0,
      cursor: 0,
      observer: pub_ch,
    };
    subscriber.connect(sub_ch);
    start_publish_loop(publisher, subscriber);
  }
}

impl<Iter, Item> SharedPublisherFactory for IterPublisherFactory<Iter, Item>
where
    Item: Send + 'static,
    Iter: IntoIterator<Item=Item> + Send + Sync + Clone + 'static,
{
  fn subscribe<S>(self, mut subscriber: S) where
      S: Subscriber<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
    let (pub_ch, sub_ch) = pub_sub_channels();
    let mut publisher = IterPublisher {
      it: self.0,
      cursor: 0,
      observer: pub_ch,
    };
    start_publish_loop(publisher, subscriber);
  }
}

impl<Iter, Item> Source for IterPublisher<Iter, Item>
  where
      Item: Send + 'static,
      Iter: IntoIterator<Item=Item> + Send + Sync + Clone + 'static
{
  type Item = Iter::Item;
  type Err = ();

  fn get_channel(&self) -> &PublisherChannel<Self::Item, Self::Err> {
    &self.observer
  }
}

impl<Iter, Item> SubscriptionLike for IterPublisher<Iter, Item>
where
    Item: Send + 'static,
    Iter: IntoIterator<Item=Item> + Clone,
{
  fn request(&mut self, requested: usize) {
    println!("{:?}", "iter req");
    let mut it = self.it.clone().into_iter().skip(self.cursor as usize);
    let mut provided = 0;
    let mut v: Option<Iter::Item>;
    loop {
      println!("{:?}", "do");
      v = it.next();
      if v.is_some() {
        self.observer.next(v.unwrap());
        provided += 1;
      } else {
        self.observer.complete();
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
) -> ObservableBase<IterPublisherFactory<Take<Repeat<Item>>, Item>>
where
  Item: Clone,
{
  from_iter(std::iter::repeat(v).take(n))
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use bencher::Bencher;
  use std::rc::Rc;
  use std::cell::RefCell;
  use std::sync::{Arc, Mutex};

  #[test]
  fn from_range() {
    /*
    let mut hit_count = 0;
    let mut completed = false;
    observable::from_iter(0..100)
      .subscribe_complete(|_| hit_count += 1, || completed = true);

    assert_eq!(hit_count, 100);
    assert!(completed);

     */
  }

  #[test]
  fn from_vec() {
    let mut hit_count = Arc::new(Mutex::new(0));
    let mut completed = Arc::new(Mutex::new(false));
    let h_c = hit_count.clone();
    let com = completed.clone();
    observable::from_iter(vec![0; 100])
      .subscribe_complete(move |_| *h_c.lock().unwrap() += 1, move || *com.lock().unwrap() = true);

    assert_eq!(*hit_count.lock().unwrap(), 100);
    assert!(*completed.lock().unwrap());
  }

  #[test]
  fn repeat_three_times() {
    /*
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

     */
  }

  #[test]
  fn repeat_zero_times() {/*
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
    */
  }
  #[test]
  fn bench() { do_bench(); }

  benchmark_group!(do_bench, bench_from_iter);

  fn bench_from_iter(b: &mut Bencher) { b.iter(from_range); }
}
