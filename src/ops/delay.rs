use crate::prelude::*;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use crate::subscriber::Subscriber;

#[derive(Clone)]
pub struct DelayOp<S, SD> {
  pub(crate) source: S,
  pub(crate) delay: Duration,
  pub(crate) scheduler: SD,
}

observable_proxy_impl!(DelayOp, S, SD);

pub struct DelayOpSubscription<S> {
  source: S,
  requested: Arc<RwLock<usize>>,
  started: bool,
  handle: SpawnHandle,
}

impl<S> SubscriptionLike for DelayOpSubscription<S>
where
  S: SubscriptionLike,
{
  fn request(&mut self, requested: usize) {
    if !self.started {
      *self.requested.write().unwrap() += requested;
      self.started = true;
    } else {
      self.source.request(requested);
    }
  }

  fn unsubscribe(&mut self) { self.handle.unsubscribe(); }

  fn is_closed(&self) -> bool { self.handle.is_closed() }
}

impl<S, SD> SharedObservable for DelayOp<S, SD>
where
  S: SharedObservable + Send + Sync + 'static,
  SD: SharedScheduler + Send + Sync + 'static,
{
  fn actual_subscribe<
    O: Subscriber<Item = Self::Item, Err = Self::Err> + Sync + Send + 'static,
  >(
    self,
    subscriber: O,
  ) {
    let delay = self.delay;
    let source = self.source;
    let scheduler = self.scheduler;
    let subscription = subscriber.subscription.clone();
    let c_subscription = subscription.clone();
    let requested = Arc::new(RwLock::new(0));
    let r_c = requested.clone();
    let handle = scheduler.schedule(
      move |_| {
        let mut sub = source.actual_subscribe(subscriber);
        let r = *r_c.read().unwrap();
        if r > 0 {
          sub.request(r);
        }
        c_subscription.add(sub);
      },
      Some(delay),
      (),
    );
    SharedSubscription::new(DelayOpSubscription {
      source: subscription,
      requested,
      started: false,
      handle,
    });
  }
}

impl<S, SD> LocalObservable<'static> for DelayOp<S, SD>
where
  S: LocalObservable<'static> + 'static,
  SD: LocalScheduler + 'static,
{
  fn actual_subscribe<O: Subscriber<Item = Self::Item, Err = Self::Err> + 'static>(
    self,
    subscriber: O,
  ) {
    let delay = self.delay;
    let source = self.source;
    let scheduler = self.scheduler;
    let subscription = subscriber.subscription.clone();
    let c_subscription = subscription.clone();
    let requested = Arc::new(RwLock::new(0));
    let r_c = requested.clone();
    let handle = scheduler.schedule(
      move |_| {
        let mut sub = source.actual_subscribe(subscriber);
        let r = *r_c.read().unwrap();
        if r > 0 {
          sub.request(r);
        }
        c_subscription.add(sub);
      },
      Some(delay),
      (),
    );
    LocalSubscription::new(DelayOpSubscription {
      source: subscription,
      requested,
      started: false,
      handle,
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use futures::executor::{LocalPool, ThreadPool};
  use std::time::Instant;
  use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
  };

  #[test]
  fn shared_smoke() {
    let value = Arc::new(Mutex::new(0));
    let c_value = value.clone();
    let pool = ThreadPool::new().unwrap();
    let stamp = Instant::now();
    observable::of(1)
      .delay(Duration::from_millis(50), pool)
      .into_shared()
      .subscribe_blocking(move |v| {
        *value.lock().unwrap() = v;
      });
    assert!(stamp.elapsed() > Duration::from_millis(50));
    assert_eq!(*c_value.lock().unwrap(), 1);
  }

  #[test]
  fn local_smoke() {
    let value = Rc::new(RefCell::new(0));
    let c_value = value.clone();
    let mut pool = LocalPool::new();
    observable::of(1)
      .delay(Duration::from_millis(50), pool.spawner())
      .subscribe(move |v| {
        *c_value.borrow_mut() = v;
      });
    assert_eq!(*value.borrow(), 0);
    let stamp = Instant::now();
    pool.run();
    assert!(stamp.elapsed() > Duration::from_millis(50));
    assert_eq!(*value.borrow(), 1);
  }
}
