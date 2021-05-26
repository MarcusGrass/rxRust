use crate::prelude::*;
use crate::subscriber::Sub;
use std::rc::Rc;

/// Creates an observable producing a multiple values.
///
/// Completes immediately after emitting the values given. Never emits an error.
///
/// # Arguments
///
/// * `v` - A value to emits.
///
/// # Examples
///
/// ```
/// use rxrust::prelude::*;
/// use rxrust::of_sequence;
///
/// of_sequence!(1, 2, 3)
///   .subscribe(|v| {println!("{},", v)});
///
/// // print log:
/// // 1
/// // 2
/// // 3
/// ```
#[macro_export]
macro_rules! of_sequence {
    ( $( $item:expr ),* ) => {
  {
    $crate::observable::create(|mut s| {
      $(
        s.next($item);
      )*
      s.complete();
    })
  }
}
}

/// Creates an observable producing a single value.
///
/// Completes immediately after emitting the value given. Never emits an error.
///
/// # Arguments
///
/// * `v` - A value to emits.
///
/// # Examples
///
/// ```
/// use rxrust::prelude::*;
///
/// observable::of(123)
///   .subscribe(|v| {println!("{},", v)});
/// ```
pub fn of<Item>(v: Item) -> ObservableBase<OfPublisherFactory<Item>> {
  ObservableBase::new(OfPublisherFactory(v))
}

#[derive(Clone)]
pub struct OfPublisherFactory<Item>(Item);

impl<Item> PublisherFactory for OfPublisherFactory<Item> {
  type Item = Item;
  type Err = ();
}

impl<'a, Item: 'a + Clone> LocalPublisherFactory<'a> for OfPublisherFactory<Item> {
  fn subscribe<S>(self, subscriber: S) where
      S: Sub<Item=Self::Item, Err=Self::Err> + 'a {
    let mut publisher = Rc::new(LocalOfPublisher(self.0, subscriber));
    publisher.1.on_subscribe(Rc::downgrade(&publisher.clone()))
  }
}

#[derive(Clone)]
struct LocalOfPublisher<Item: Clone, O>(Item, O);

impl<Item: Clone, O> SubscriptionLike for LocalOfPublisher<Item, O>
where
  O: Sub<Item = Item>,
{
  fn request(&mut self, _: usize) {
    self.1.next(self.0.clone());
    self.1.complete();
  }

  fn unsubscribe(&mut self) {}

  fn is_closed(&self) -> bool { todo!() }
}

impl<Item: Clone + Sync + Send, O> SubscriptionLike for SharedOfPublisher<Item, O>
where
  O: Sub<Item = Item> + Send + Sync + 'static,
{
  fn request(&mut self, _: usize) {
    self.1.next(self.0.clone());
    self.1.complete();
  }

  fn unsubscribe(&mut self) { todo!() }

  fn is_closed(&self) -> bool { todo!() }
}

#[derive(Clone)]
struct SharedOfPublisher<Item, O>(Item, O);

impl<Item: Clone + Sync + Send + 'static> SharedPublisherFactory
  for OfPublisherFactory<Item>
{
  fn subscribe<S>(self, subscriber: S) where
      S: Sub<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
    let mut publisher = Rc::new(SharedOfPublisher(self.0, subscriber));
    publisher.1.on_subscribe(Rc::downgrade(&publisher.clone()));
  }
}

/// Creates an observable that emits value or the error from a [`Result`] given.
///
/// Completes immediately after.
///
/// # Arguments
///
/// * `r` - A [`Result`] argument to take a value, or an error to emits from.
///
/// # Examples
///
/// ```
/// use rxrust::prelude::*;
///
/// observable::of_result(Ok(1234))
///   .subscribe(|v| {println!("{},", v)});
/// ```
///
/// ```
/// use rxrust::prelude::*;
///
/// observable::of_result(Err("An error"))
///   .subscribe_err(|v: &i32| {}, |e| {println!("Error:  {},", e)});
/// ```
pub fn of_result<Item, Err>(
  r: Result<Item, Err>,
) -> ObservableBase<OfResultPublisherFactory<Item, Err>> {
  ObservableBase::new(OfResultPublisherFactory(r))
}

#[derive(Clone)]
pub struct OfResultPublisherFactory<Item, Err>(Result<Item, Err>);

impl<Item, Err> PublisherFactory for OfResultPublisherFactory<Item, Err> {
  type Item = Item;
  type Err = Err;
}

impl<'a, Item: Clone + 'a, Err: Clone + 'a> LocalPublisherFactory<'a>
  for OfResultPublisherFactory<Item, Err>
{
  fn subscribe<S>(self, subscriber: S) where
      S: Sub<Item=Self::Item, Err=Self::Err> + 'a {
    let mut publisher = Rc::new(LocalOfResultPublisher(self.0, subscriber));
    publisher.1.on_subscribe(Rc::downgrade(&publisher.clone()))
  }
}

#[derive(Clone)]
struct LocalOfResultPublisher<Item, Err, O>(
  Result<Item, Err>,
  O,
);

impl<Item: Clone, Err: Clone, O> SubscriptionLike
  for LocalOfResultPublisher<Item, Err, O>
where
  O: Sub<Item = Item, Err = Err>,
{
  fn request(&mut self, _: usize) {
    match self.0.clone() {
      Ok(v) => {
        self.1.next(v);
        self.1.complete();
      }
      Err(e) => self.1.error(e),
    }
  }

  fn unsubscribe(&mut self) { todo!() }

  fn is_closed(&self) -> bool { todo!() }
}

impl<Item: Clone + Send + Sync + 'static, Err: Clone + Send + Sync + 'static, O>
  SubscriptionLike for SharedOfResultPublisher<Item, Err, O>
where
  O: Sub<Item = Item, Err = Err> + Send + Sync + 'static,
{
  fn request(&mut self, _: usize) {
    match self.0.clone() {
      Ok(v) => {
        self.1.next(v);
        self.1.complete();
      }
      Err(e) => self.1.error(e),
    }
  }

  fn unsubscribe(&mut self) { todo!() }

  fn is_closed(&self) -> bool { todo!() }
}

#[derive(Clone)]
struct SharedOfResultPublisher<Item, Err, O>(
  Result<Item, Err>,
  O
);

impl<Item: Clone + Send + Sync + 'static, Err: Clone + Send + Sync + 'static>
  SharedPublisherFactory for OfResultPublisherFactory<Item, Err>
{

  fn subscribe<S>(self, subscriber: S) where
      S: Sub<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
    let mut publisher = Rc::new(SharedOfResultPublisher(self.0, subscriber));
    publisher.1.on_subscribe(Rc::downgrade(&publisher.clone()));
  }
}

/// Creates an observable that potentially emits a single value from [`Option`].
///
/// Emits the value if is there, and completes immediately after. When the
/// given option has not value, completes immediately. Never emits an error.
///
/// # Arguments
///
/// * `o` - An optional used to take a value to emits from.
///
/// # Examples
///
/// ```
/// use rxrust::prelude::*;
///
/// observable::of_option(Some(1234))
///   .subscribe(|v| {println!("{},", v)});
/// ```
pub fn of_option<Item>(
  o: Option<Item>,
) -> ObservableBase<OfOptionPublisherFactory<Item>> {
  ObservableBase::new(OfOptionPublisherFactory(o))
}
#[derive(Clone)]
pub struct OfOptionPublisherFactory<Item>(Option<Item>);

impl<Item> PublisherFactory for OfOptionPublisherFactory<Item> {
  type Item = Item;
  type Err = ();
}

impl<'a, Item: Clone + 'a> LocalPublisherFactory<'a> for OfOptionPublisherFactory<Item> {

  fn subscribe<S>(self, subscriber: S) where
      S: Sub<Item=Self::Item, Err=Self::Err> + 'a {
    let mut publisher = Rc::new(LocalOfOptionPublisher(self.0, subscriber));
    publisher.1.on_subscribe(Rc::downgrade(&publisher.clone()))
  }
}

#[derive(Clone)]
struct LocalOfOptionPublisher<Item, O>(
  Option<Item>,
  O
);

impl<Item: Clone, O> SubscriptionLike for LocalOfOptionPublisher<Item, O>
where
  O: Sub<Item = Item>,
{
  fn request(&mut self, _: usize) {
    match self.0.clone() {
      Some(v) => self.1.next(v),
      None => {}
    };
    self.1.complete();
  }

  fn unsubscribe(&mut self) { todo!() }

  fn is_closed(&self) -> bool { todo!() }
}

impl<Item: Clone + Send + Sync + 'static, O> SubscriptionLike
  for SharedOfOptionPublisher<Item, O>
where
  O: Sub<Item = Item> + Send + Sync + 'static,
{
  fn request(&mut self, _: usize) {
    match self.0.clone() {
      Some(v) => self.1.next(v),
      None => {}
    };
    self.1.complete();
  }

  fn unsubscribe(&mut self) { todo!() }

  fn is_closed(&self) -> bool { todo!() }
}

#[derive(Clone)]
struct SharedOfOptionPublisher<Item, O>(Option<Item>, O);

impl<Item: Clone + Send + Sync + 'static> SharedPublisherFactory
  for OfOptionPublisherFactory<Item>
{

  fn subscribe<S>(self, subscriber: S) where
      S: Sub<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
    let mut publisher = Rc::new(SharedOfOptionPublisher(self.0, subscriber));
    publisher.1.on_subscribe(Rc::downgrade(&publisher.clone()))
  }
}

/// Creates an observable that emits the return value of a callable.
///
/// Never emits an error.
///
/// # Arguments
///
/// * `f` - A function that will be called to obtain its return value to emits.
///
/// # Examples
///
/// ```
/// use rxrust::prelude::*;
///
/// observable::of_fn(|| {1234})
///   .subscribe(|v| {println!("{},", v)});
/// ```
pub fn of_fn<F, Item>(f: F) -> ObservableBase<OfFnPublisherFactory<F, Item>> {
  ObservableBase::new(OfFnPublisherFactory(f, TypeHint::new()))
}

#[derive(Clone)]
pub struct OfFnPublisherFactory<F, Item>(F, TypeHint<Item>);

impl<F, Item> PublisherFactory for OfFnPublisherFactory<F, Item> {
  type Item = Item;
  type Err = ();
}

impl<'a, F: 'a, Item: 'a> LocalPublisherFactory<'a> for OfFnPublisherFactory<F, Item>
where
  F: Fn() -> Item,
{

  fn subscribe<S>(self, subscriber: S) where
      S: Sub<Item=Self::Item, Err=Self::Err> + 'a {
    let mut publisher = Rc::new(LocalOfFnPublisher(self.0, subscriber, TypeHint::new()));
    publisher.1.on_subscribe(Rc::downgrade(&publisher.clone()));
  }
}

#[derive(Clone)]
struct LocalOfFnPublisher<F, Item, O>(
  F,
  O,
  TypeHint<Item>,
);

impl<F, Item, O> SubscriptionLike for LocalOfFnPublisher<F, Item, O>
where
  O: Sub<Item = Item>,
  F: Fn() -> Item,
{
  fn request(&mut self, _: usize) {
    self.1.next((self.0)());
    self.1.complete();
  }

  fn unsubscribe(&mut self) { todo!() }

  fn is_closed(&self) -> bool { todo!() }
}

impl<F, Item, O> SubscriptionLike for SharedOfFnPublisher<F, Item, O>
where
  O: Sub<Item = Item> + Send + Sync + 'static,
  F: Fn() -> Item,
{
  fn request(&mut self, _: usize) {
    self.1.next((self.0)());
    self.1.complete();
  }

  fn unsubscribe(&mut self) { todo!() }

  fn is_closed(&self) -> bool { todo!() }
}

#[derive(Clone)]
struct SharedOfFnPublisher<F, Item, O>(
  F,
  O,
  TypeHint<Item>,
);

impl<F, Item: 'static> SharedPublisherFactory for OfFnPublisherFactory<F, Item>
where
  F: Fn() -> Item + Send + Sync + 'static,
{

  fn subscribe<S>(self, subscriber: S) where
      S: Sub<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
    let mut publisher = Rc::new(SharedOfFnPublisher(self.0, subscriber, TypeHint::new()));
    publisher.1.on_subscribe(Rc::downgrade(&publisher.clone()))
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn from_fn() {
    let mut value = 0;
    let mut completed = false;
    let callable = || 123;
    observable::of_fn(callable).subscribe_complete(
      |v| {
        value = v;
      },
      || completed = true,
    );

    assert_eq!(value, 123);
    assert!(completed);
  }

  #[test]
  fn of_option() {
    let mut value1 = 0;
    let mut completed1 = false;
    observable::of_option(Some(123)).subscribe_complete(
      |v| {
        value1 = v;
      },
      || completed1 = true,
    );

    assert_eq!(value1, 123);
    assert!(completed1);

    let mut value2 = 0;
    let mut completed2 = false;
    observable::of_option(None).subscribe_complete(
      |v| {
        value2 = v;
      },
      || completed2 = true,
    );

    assert_eq!(value2, 0);
    assert!(completed2);
  }

  #[test]
  fn of_result() {
    let mut value1 = 0;
    let mut completed1 = false;
    let r: Result<i32, &str> = Ok(123);
    observable::of_result(r).subscribe_all(
      |v| {
        value1 = v;
      },
      |_| {},
      || completed1 = true,
    );

    assert_eq!(value1, 123);
    assert!(completed1);

    let mut value2 = 0;
    let mut error_reported = false;
    let r: Result<i32, &str> = Err("error");
    observable::of_result(r).subscribe_err(|_| value2 = 123, |_| error_reported = true);

    assert_eq!(value2, 0);
    assert!(error_reported);
  }

  #[test]
  fn of() {
    let mut value = 0;
    let mut completed = false;
    observable::of(100).subscribe_complete(|v| value = v, || completed = true);

    assert_eq!(value, 100);
    assert!(completed);
  }

  #[test]
  fn of_macros() {
    let mut value = 0;
    of_sequence!(1, 2, 3).subscribe(|v| value += v);

    assert_eq!(value, 6);
  }

  #[test]
  fn bench() { do_bench(); }

  benchmark_group!(do_bench, bench_of);

  fn bench_of(b: &mut bencher::Bencher) { b.iter(of); }
}
