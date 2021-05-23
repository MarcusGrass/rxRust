use crate::prelude::*;

/// Creates an observable that will on subscription defer to another observable
/// that is supplied by a supplier-function which will be run once at each
/// subscription
///
/// ```rust
/// # use rxrust::prelude::*;
///
/// observable::defer(|| {
///   println!("Hi!");
///   observable::of("Hello!")
/// })
///   .subscribe(move |v| {
///     println!("{}", v);
///   });
/// // Prints: Hi!\nHello!\n
/// ```
pub fn defer<F, Item, Err, Emit>(
  observable_supplier: F,
) -> ObservableBase<DeferPublisherFactory<F, Emit, Item, Err>>
where
  F: FnOnce() -> ObservableBase<Emit>,
  Emit: PublisherFactory,
{
  ObservableBase::new(DeferPublisherFactory(observable_supplier, TypeHint::new()))
}

#[derive(Clone)]
struct DeferPublisherFactory<F, Emit, Item, Err>(F, TypeHint<(Item, Err, Emit)>);

impl<F, Emit, Item, Err> PublisherFactory for DeferPublisherFactory<F, Emit, Item, Err> {
  type Item = Item;
  type Err = Err;
}

impl<'a, F, Emit, Item, Err> LocalPublisherFactory<'a> for DeferPublisherFactory<F, Emit, Item, Err> {
  fn subscribe<O>(self, subscriber: Subscriber<O, LocalSubscription<'a>>) -> LocalSubscription<'a> where
      O: Observer<Item=Self::Item, Err=Self::Err> + 'a {
    todo!()
  }
}

#[derive(Clone)]
struct LocalDeferPublisher<'a, F, O, Emit> {
  func: F,
  sub: Subscriber<O, LocalSubscription<'a>>,
  _h: TypeHint<Emit>

}

impl<'a, F, O, Emit> SubscriptionLike for LocalDeferPublisher<'a, F, O, Emit>
where F: FnOnce() -> ObservableBase<Emit>
{
  fn request(&mut self, _: u128) {
    (self.func)();
  }

  fn unsubscribe(&mut self) {
    todo!()
  }

  fn is_closed(&self) -> bool {
    todo!()
  }
}

impl<F, Item, Err, Emit> SharedPublisherFactory for DeferPublisherFactory<F, Item, Err, Emit> {
  fn subscribe<O>(self, subscriber: Subscriber<O, SharedSubscription>) -> SharedSubscription where
      O: Observer<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
    todo!()
  }
}

#[derive(Clone)]
struct SharedDeferPublisher<F, O, Emit> {
  func: F,
  sub: Subscriber<O, SharedSubscription>,
  _h: TypeHint<Emit>
}

impl<'a, F, O, Emit> SubscriptionLike for SharedDeferPublisher< F, O, Emit>
  where F: FnOnce() -> ObservableBase<Emit>
{
  fn request(&mut self, _: u128) {
    (self.func)();
  }

  fn unsubscribe(&mut self) {
    todo!()
  }

  fn is_closed(&self) -> bool {
    todo!()
  }
}


#[cfg(test)]
mod test {
  use std::ops::Deref;
  use std::sync::{Arc, Mutex};

  use crate::prelude::*;
  use bencher::Bencher;

  #[test]
  fn no_results_before_deferred_subscribe() {
    let calls = Arc::new(Mutex::new(0));
    let sum = Arc::new(Mutex::new(0));
    let errs = Arc::new(Mutex::new(0));
    let completes = Arc::new(Mutex::new(0));

    let deferred = observable::defer(|| {
      *calls.lock().unwrap() += 1;
      observable::of(&2)
    })
    .into_shared();

    assert_eq!(calls.lock().unwrap().deref(), &0);

    for i in 1..4 {
      let sum_copy = Arc::clone(&sum);
      let errs_copy = Arc::clone(&errs);
      let completes_copy = Arc::clone(&completes);
      deferred.clone().subscribe_all(
        move |v| *sum_copy.lock().unwrap() += v,
        move |_| *errs_copy.lock().unwrap() += 1,
        move || *completes_copy.lock().unwrap() += 1,
      );
      assert_eq!(*calls.lock().unwrap(), i);
    }

    assert_eq!(*calls.lock().unwrap().deref(), 3);
    assert_eq!(*sum.lock().unwrap().deref(), 6);
    assert_eq!(*errs.lock().unwrap().deref(), 0);
    assert_eq!(*completes.lock().unwrap().deref(), 3);
  }

  #[test]
  fn support_fork() {
    let calls = Arc::new(Mutex::new(0));
    let o = observable::defer(|| {
      *calls.lock().unwrap() += 1;
      observable::of(10)
    });
    let sum1 = Arc::new(Mutex::new(0));
    let sum2 = Arc::new(Mutex::new(0));
    let c_sum1 = sum1.clone();
    let c_sum2 = sum2.clone();
    o.clone().subscribe(move |v| *sum1.lock().unwrap() += v);
    o.clone().subscribe(move |v| *sum2.lock().unwrap() += v);

    assert_eq!(*c_sum1.lock().unwrap(), 10);
    assert_eq!(*c_sum2.lock().unwrap(), 10);
    assert_eq!(*calls.lock().unwrap().deref(), 2);
  }

  #[test]
  fn fork_and_share() {
    let observable = observable::defer(|| observable::empty());
    observable.clone().into_shared().subscribe(|_: i32| {});
    observable.clone().into_shared().subscribe(|_| {});

    let observable = observable::defer(|| observable::empty()).into_shared();
    observable.clone().subscribe(|_: i32| {});
    observable.clone().subscribe(|_| {});
  }

  #[test]
  fn bench() { do_bench(); }

  benchmark_group!(do_bench, bench_deref);

  fn bench_deref(b: &mut Bencher) {
    b.iter(no_results_before_deferred_subscribe);
  }
}
