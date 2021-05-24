use crate::prelude::*;
use crate::scheduler::SharedScheduler;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone)]
pub struct SubscribeOnOP<S, SD> {
  pub(crate) source: S,
  pub(crate) scheduler: SD,
}

observable_proxy_impl!(SubscribeOnOP, S, SD);

impl<'a, S, SD> SharedObservable for SubscribeOnOP<S, SD>
where
  S: SharedObservable + Send + 'static,
  SD: SharedScheduler + Send + 'static,
  S::Unsub: Send + Sync,
{
  type Unsub = SharedSubscription;
  fn actual_subscribe<
    O: Observer<Item = Self::Item, Err = Self::Err> + Sync + Send + 'static,
  >(
    self,
    subscriber: Subscriber<O, SharedSubscription>,
  ) -> Self::Unsub {
    let source = self.source;
    let subscription = subscriber.subscription.clone();
    let req = Arc::new(RwLock::new(0));
    let r_c = req.clone();
    let handle = self.scheduler.schedule(
      move |_| {
        let subscription = subscriber.subscription.clone();
        let mut actual = source.actual_subscribe(subscriber);
        let r = *r_c.read().unwrap();
        if r > 0 {
          actual.request(r);
        }
        subscription.add(actual);
      },
      None,
      (),
    );
    SharedSubscription::new(OnSubSubscription{
      sub: subscription,
      handle,
      req,
      started: false
    })
  }
}

#[derive(Clone)]
pub struct OnSubSubscription<S> {
  sub: S,
  handle: SpawnHandle,
  req: Arc<RwLock<u128>>,
  started: bool,
}

impl<S> SubscriptionLike for OnSubSubscription<S> where S: SubscriptionLike {
  fn request(&mut self, requested: u128) {
    if !self.started {
      *self.req.write().unwrap() += requested;
      self.started = true;
    } else {
      self.sub.request(requested);
    }
  }

  fn unsubscribe(&mut self) {
    self.handle.unsubscribe();
  }

  fn is_closed(&self) -> bool {
    self.handle.is_closed()
  }
}

impl<S, SD> LocalObservable<'static> for SubscribeOnOP<S, SD>
where
  S: LocalObservable<'static> + 'static,
  SD: LocalScheduler,
{
  type Unsub = LocalSubscription<'static>;
  fn actual_subscribe<
    O: Observer<Item = Self::Item, Err = Self::Err> + 'static,
  >(
    self,
    subscriber: Subscriber<O, LocalSubscription<'static>>,
  ) -> Self::Unsub {
    let source = self.source;
    let subscription = subscriber.subscription.clone();
    let handle = self.scheduler.schedule(
      move |_| {
        let subscription = subscriber.subscription.clone();
        subscription.add(source.actual_subscribe(subscriber))
      },
      None,
      (),
    );
    subscription.add(handle);
    subscription
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use futures::executor::ThreadPool;
  use std::sync::{Arc, Mutex};
  use std::thread;
  use std::time::Duration;

  #[test]
  fn thread_pool() {
    let pool = ThreadPool::new().unwrap();
    let res = Arc::new(Mutex::new(vec![]));
    let c_res = res.clone();
    let thread = Arc::new(Mutex::new(vec![]));
    let c_thread = thread.clone();
    observable::from_iter(1..5)
      .subscribe_on(pool)
      .into_shared()
      .subscribe_blocking(move |v| {
        res.lock().unwrap().push(v);
        let handle = thread::current();
        thread.lock().unwrap().push(handle.id());
      });

    thread::sleep(std::time::Duration::from_millis(1));
    assert_eq!(*c_res.lock().unwrap(), (1..5).collect::<Vec<_>>());
    assert_ne!(c_thread.lock().unwrap()[0], thread::current().id());
  }

  #[test]
  fn pool_unsubscribe() {
    let pool = ThreadPool::new().unwrap();
    let emitted = Arc::new(Mutex::new(vec![]));
    let c_emitted = emitted.clone();
    observable::from_iter(0..10)
      .subscribe_on(pool.clone())
      .delay(Duration::from_millis(10), pool)
      .into_shared()
      .subscribe(move |v| {
        emitted.lock().unwrap().push(v);
      })
      .unsubscribe();
    std::thread::sleep(Duration::from_millis(20));
    assert_eq!(c_emitted.lock().unwrap().len(), 0);
  }
}
