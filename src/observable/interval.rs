use crate::prelude::*;

use std::time::{Duration, Instant};
use std::sync::{Arc, RwLock};

/// Creates an observable which will fire at `dur` time into the future,
/// and will repeat every `dur` interval after.
pub fn interval<S>(
  dur: Duration,
  scheduler: S,
) -> ObservableBase<IntervalPublisherFactory<S>> {
  ObservableBase::new(IntervalPublisherFactory {
    dur,
    at: None,
    scheduler,
  })
}

/// Creates an observable which will fire at the time specified by `at`,
/// and then will repeat every `dur` interval after
pub fn interval_at<S>(
  at: Instant,
  dur: Duration,
  scheduler: S,
) -> ObservableBase<IntervalPublisherFactory<S>> {
  ObservableBase::new(IntervalPublisherFactory {
    scheduler,
    dur,
    at: Some(at),
  })
}

#[derive(Clone)]
pub struct IntervalPublisherFactory<S> {
  scheduler: S,
  dur: Duration,
  at: Option<Instant>,
}

impl<S> PublisherFactory for IntervalPublisherFactory<S> {
  type Item = u128;
  type Err = ();
}

impl<S> LocalPublisherFactory<'static> for IntervalPublisherFactory<S>
where S: LocalScheduler
{
  fn subscribe<O>(self, subscriber: Subscriber<O, LocalSubscription<'static>>) -> LocalSubscription<'static> where
      O: Observer<Item=Self::Item, Err=Self::Err> + 'static
  {
    let mut observer = subscriber.observer;
    let requested = Arc::new(RwLock::new(0));
    let r_c = requested.clone();
    let handle = self.scheduler.schedule_repeating(
      move |i| {
        if *r_c.read().unwrap() > 0 {
          observer.next(i as u128);
        }
      },
      self.dur,
      self.at,
    );
    println!("{:?}", "init sub");
    LocalSubscription::new(IntervalPublisher {
      dur: self.dur,
      at: self.at,
      requested,
      abort: handle
    })
  }
}

impl<S> SharedPublisherFactory for IntervalPublisherFactory<S>
where S: SharedScheduler
{
  fn subscribe<O>(self, subscriber: Subscriber<O, SharedSubscription>) -> SharedSubscription where
      O: Observer<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static
  {
    let mut observer = subscriber.observer;
    let requested = Arc::new(RwLock::new(0));
    let r_c = requested.clone();
    let handle = self.scheduler.schedule_repeating(
      move |i| {
        if *r_c.read().unwrap() > 0 { // TODO: This will by default onBackpressureDrop emissions if none are requested, could cache them instead
          observer.next(i as u128);
          *r_c.write().unwrap() -= 1;
        }
      },
      self.dur,
      self.at,
    );
    SharedSubscription::new(IntervalPublisher {
      dur: self.dur,
      at: self.at,
      requested,
      abort: handle
    })
  }
}

struct IntervalPublisher {
  dur: Duration,
  at: Option<Instant>,
  requested: Arc<RwLock<u128>>,
  abort: SpawnHandle
}


impl SubscriptionLike for IntervalPublisher {
  fn request(&mut self, requested: u128) {
    println!("{:?}", "req");
    *self.requested.write().unwrap() += requested;
  }

  fn unsubscribe(&mut self) {
    println!("{:?}", "unsub");
    self.abort.unsubscribe();
  }

  fn is_closed(&self) -> bool {
    self.abort.is_closed()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_scheduler::ManualScheduler;
  use futures::executor::{LocalPool, ThreadPool};
  use std::sync::{Arc, Mutex};

  #[test]
  fn shared() {
    let millis = Arc::new(Mutex::new(0));
    let c_millis = millis.clone();
    let stamp = Instant::now();
    let pool = ThreadPool::new().unwrap();

    interval(Duration::from_millis(1), pool)
      .take(5) // Will block forever if we don't limit emissions
      .into_shared()
      .subscribe_blocking(move |_| {
        *millis.lock().unwrap() += 1;
      });

    assert_eq!(*c_millis.lock().unwrap(), 5);
    assert!(stamp.elapsed() > Duration::from_millis(5));
  }

  #[test]
  fn local() {
    let mut local = LocalPool::new();
    let stamp = Instant::now();
    let ticks = Arc::new(Mutex::new(0));
    let ticks_c = Arc::clone(&ticks);
    println!("{:?}", "Start");
    interval(Duration::from_millis(1), local.spawner())
      .take(5)
      .subscribe(move |_| (*ticks_c.lock().unwrap()) += 1);
    println!("{:?}", "end");
    local.run();
    assert_eq!(*ticks.lock().unwrap(), 5);
    assert!(stamp.elapsed() > Duration::from_millis(5));
  }

  #[test]
  fn local_manual() {
    let scheduler = ManualScheduler::now();
    let ticks = Arc::new(Mutex::new(0));
    let ticks_c = Arc::clone(&ticks);
    let delay = Duration::from_millis(1);
    interval(delay, scheduler.clone())
      .take(5)
      .subscribe(move |_| (*ticks_c.lock().unwrap()) += 1);
    assert_eq!(0, *ticks.lock().unwrap());
    scheduler.advance(delay * 2);
    scheduler.run_tasks();
    assert_eq!(2, *ticks.lock().unwrap());

    scheduler.advance(delay * 3);
    scheduler.run_tasks();
    assert_eq!(5, *ticks.lock().unwrap());
  }
}
