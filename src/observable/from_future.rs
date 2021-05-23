use crate::prelude::*;
use futures::{FutureExt};
use std::future::Future;

/// Converts a `Future` to an observable sequence. Even though if the future
/// poll value has `Result::Err` type, also emit as a normal value, not trigger
/// to error handle.
///
/// ```rust
/// # use rxrust::prelude::*;
/// use futures::{future, executor::LocalPool};
/// let mut local_scheduler = LocalPool::new();
///
/// observable::from_future(future::ready(1), local_scheduler.spawner())
///   .subscribe(move |v| {
///     println!("subscribed {}", v);
///   });
///
/// local_scheduler.run();
/// ```
/// If your `Future` poll an `Result` type value, and you want dispatch the
/// error by rxrust, you can use [`from_future_result`]
pub fn from_future<F, Item, S>(
  f: F,
  scheduler: S,
) -> ObservableBase<FuturePublisherFactory<F, S>>
where
  F: Future<Output = Item>,
{
  ObservableBase::new(FuturePublisherFactory {
    future: f,
    scheduler,
  })
}


#[derive(Clone)]
pub struct FuturePublisherFactory<F, S> {
  future: F,
  scheduler: S,
}

#[derive(Clone)]
struct LocalFuturePublisher<F> {
  future: F,
  abort: SpawnHandle,
}

impl<F> SubscriptionLike for LocalFuturePublisher<F>
  where
      F: Future,
{
  fn request(&mut self, _: u128) {

  }

  fn unsubscribe(&mut self) {
    self.abort.unsubscribe();
  }

  fn is_closed(&self) -> bool {
    self.abort.is_closed() // TODO: Is this correct?
  }
}

#[derive(Clone)]
struct SharedFuturePublisher<F> {
  future: F,
  abort: SpawnHandle,
}

impl<F> SubscriptionLike for SharedFuturePublisher<F>
  where
      F: Future,
{
  fn request(&mut self, _: u128) {
  }

  fn unsubscribe(&mut self) {
    self.abort.unsubscribe();
  }

  fn is_closed(&self) -> bool {
    self.abort.is_closed() // TODO: Is this correct?
  }
}
impl<F, S> PublisherFactory for FuturePublisherFactory<F, S>
where F: Future
{
  type Item = F::Output;
  type Err = ();
}

impl<F, S> LocalPublisherFactory<'static> for FuturePublisherFactory<F, S>
  where F: Future + 'static, S: LocalScheduler
{
  fn subscribe<O>(self, mut subscriber: Subscriber<O, LocalSubscription<'static>>) -> LocalSubscription<'static> where
      O: Observer<Item=Self::Item, Err=Self::Err> + 'static {
    let f = self.future;
    let (future, handle) = futures::future::abortable(f);
    self.scheduler.spawn(future.map(move |v| {
      if let Ok(output) = v {
        subscriber.observer.next(output);
        subscriber.observer.complete();
      }
    }));
    let subscription = LocalSubscription::default();
    subscription.add(LocalFuturePublisher{
      future,
      abort: SpawnHandle::new(handle),
    });
    subscription
  }
}

impl<F, S> SharedPublisherFactory for FuturePublisherFactory<F, S>
  where F: Future + Send + Sync + 'static, S: SharedScheduler
{
  fn subscribe<O>(self, mut subscriber: Subscriber<O, SharedSubscription>) -> SharedSubscription where
      O: Observer<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {

    let f = self.future;
    let (future, handle) = futures::future::abortable(f);
    self.scheduler.spawn(future.map(move |v| {
      if let Ok(output) = v {
        subscriber.observer.next(output);
        subscriber.observer.complete();
      }
    }));
    let subscription = SharedSubscription::default();
    subscription.add(SharedFuturePublisher{
      future,
      abort: SpawnHandle::new(handle),
    });
    subscription
  }
}


/// Converts a `Future` to an observable sequence like
/// [`from_future@from_future`]. But only work for which `Future::Output` is
/// `Result` type, and `Result::Ok` emit to next handle, and `Result::Err` as an
/// error to handle.
pub fn from_future_result<F, S, Item, Err>(
  future: F,
  scheduler: S,
) -> ObservableBase<FutureResultPublisherFactory<F, S, Item, Err>>
where
  F: Future,
  <F as Future>::Output: Into<Result<Item, Err>>,
{
  ObservableBase::new(FutureResultPublisherFactory {
    future,
    scheduler,
    marker: TypeHint::new(),
  })
}

#[derive(Clone)]
pub struct FutureResultPublisherFactory<F, S, Item, Err> {
  future: F,
  scheduler: S,
  marker: TypeHint<(Item, Err)>,
}

impl<F, S, Item, Err> PublisherFactory for FutureResultPublisherFactory<F, S, Item, Err> {
  type Item = Item;
  type Err = Err;
}

impl<'a, F, S, Item, Err> LocalPublisherFactory<'a> for FutureResultPublisherFactory<F, S, Item, Err> where
  F: Future<Output=Self::Item> + 'a
{
  fn subscribe<O>(self, mut subscriber: Subscriber<O, LocalSubscription<'a>>) -> LocalSubscription<'a> where
      O: Observer<Item=Self::Item, Err=Self::Err> + 'a {

    let f = self.future.map(move |v| {
      subscriber.observer.next(v);
      subscriber.observer.complete();
    });
    let (future, handle) = futures::future::abortable(f);
    let subscription = LocalSubscription::default();
    subscription.add(
      LocalFutureResultPublisher{
        future,
        abort: SpawnHandle::new(handle),
      }
    );
    subscription
  }
}
impl<F, S, Item, Err> SharedPublisherFactory for FutureResultPublisherFactory<F, S, Item, Err>
where S: SharedScheduler, F: Future<Output=Self::Item> + Send + Sync + 'static
{
  fn subscribe<O>(self, mut subscriber: Subscriber<O, SharedSubscription>) -> SharedSubscription where
      O: Observer<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {

    let f = self.future.map(move |v| {
      subscriber.observer.next(v);
      subscriber.observer.complete();
    });
    let (future, handle) = futures::future::abortable(f);
    let subscription = SharedSubscription::default();
    subscription.add(
      SharedFutureResultPublisher{
        future,
        abort: SpawnHandle::new(handle),
      }
    );
    subscription
  }
}

#[derive(Clone)]
pub struct LocalFutureResultPublisher<F> {
  future: F,
  abort: SpawnHandle
}

impl<F> SubscriptionLike for LocalFutureResultPublisher<F> {
  fn request(&mut self, requested: u128) {
    todo!()
  }

  fn unsubscribe(&mut self) {
    todo!()
  }

  fn is_closed(&self) -> bool {
    todo!()
  }
}

#[derive(Clone)]
pub struct SharedFutureResultPublisher<F> {
  future: F,
  abort: SpawnHandle
}

impl<F> SubscriptionLike for SharedFutureResultPublisher<F> {
  fn request(&mut self, requested: u128) {
    todo!()
  }

  fn unsubscribe(&mut self) {
    todo!()
  }

  fn is_closed(&self) -> bool {
    todo!()
  }
}


#[cfg(test)]
mod tests {
  use super::*;
  use bencher::Bencher;
  use futures::{
    executor::{LocalPool, ThreadPool},
    future,
  };
  use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
  };

  #[test]
  fn shared() {
    let res = Arc::new(Mutex::new(0));
    let c_res = res.clone();
    let pool = ThreadPool::new().unwrap();
    // from_future
    let res = c_res.clone();
    from_future(future::ready(2), pool)
        .into_shared()
        .subscribe(move |v| {
          *res.lock().unwrap() = v;
        });
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert_eq!(*c_res.lock().unwrap(), 2);
    {
      from_future_result(future::ok(1), pool.clone())
        .into_shared()
        .subscribe(move |v| {
          *res.lock().unwrap() = v;
        });
      std::thread::sleep(std::time::Duration::from_millis(10));
      assert_eq!(*c_res.lock().unwrap(), 1);
    }

  }

  #[test]
  fn local() {
    let mut local = LocalPool::new();
    let value = Rc::new(RefCell::new(0));
    let v_c = value.clone();
    from_future_result(future::ok(1), local.spawner()).subscribe(move |v| {
      *v_c.borrow_mut() = v;
    });
    local.run();
    assert_eq!(*value.borrow(), 1);

    let v_c = value.clone();
    from_future(future::ready(2), local.spawner()).subscribe(move |v| {
      *v_c.borrow_mut() = v;
    });

    local.run();
    assert_eq!(*value.borrow(), 2);
  }

  #[test]
  fn bench() { do_bench(); }

  benchmark_group!(do_bench, bench_from_future);

  fn bench_from_future(b: &mut Bencher) { b.iter(local); }
}
