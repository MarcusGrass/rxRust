use crate::prelude::*;

use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use crate::subscriber::Subscriber;
use std::rc::Rc;

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

impl<SD> LocalPublisherFactory<'static> for IntervalPublisherFactory<SD>
where
    SD: LocalScheduler + 'static,
{

  fn subscribe(self, channel: PublisherChannel<Self::Item, Self::Err>) {
    /*
    let mut publisher = Rc::new(LocalIntervalPublisher {
      scheduler: self.scheduler,
      observer: Arc::new(RwLock::new(subscriber)),
      abort: None,
      at: self.at,
      dur: self.dur,
    });
    publisher.clone().observer.write().unwrap().connect(LocalSubscription::new(publisher));

     */
  }
}

impl<SD> SharedPublisherFactory for IntervalPublisherFactory<SD>
where
    SD: SharedScheduler + Send + Sync + 'static,
{

  fn subscribe<S>(self, subscriber: S) where
      S: Subscriber<Item=Self::Item, Err=Self::Err> + Send + Sync + 'static {
    /*
    let mut publisher = Arc::new(SharedIntervalPublisher {
      scheduler: self.scheduler,
      observer: Arc::new(RwLock::new(subscriber)),
      abort: None,
      at: self.at,
      dur: self.dur,
    });
    publisher.clone().observer.write().unwrap().connect(SharedSubscription::new(publisher));

     */
  }
}

#[derive(Clone)]
struct LocalIntervalPublisher<S, O> {
  scheduler: S,
  observer: Arc<RwLock<O>>,
  abort: Option<SpawnHandle>,
  at: Option<Instant>,
  dur: Duration,
}

impl<S, O> SubscriptionLike for LocalIntervalPublisher<S, O>
where
  S: LocalScheduler,
  O: Observer<Item = u128> + 'static,
{
  fn request(&mut self, requested: usize) {
    let o_c = self.observer.clone();
    let h = self.scheduler.schedule_repeating(
      move |i| {
        o_c.write().unwrap().next(i as u128);
      },
      self.dur,
      self.at,
      Some(requested as usize),
    );
    if self.abort.is_some() {
      self.abort.clone().unwrap().unsubscribe();
    }
    self.abort = Some(h);
  }

  fn unsubscribe(&mut self) {
    if self.abort.is_some() {
      self.abort.clone().unwrap().unsubscribe();
    }
  }

  fn is_closed(&self) -> bool {
    if self.abort.is_some() {
      return self.abort.clone().unwrap().is_closed();
    }
    true // TODO: true if not started
  }
}
#[derive(Clone)]
struct SharedIntervalPublisher<S, O> {
  scheduler: S,
  observer: Arc<RwLock<O>>,
  abort: Option<SpawnHandle>,
  at: Option<Instant>,
  dur: Duration,
}

impl<S, O> SubscriptionLike for SharedIntervalPublisher<S, O>
where
  S: SharedScheduler,
  O: Observer<Item = u128> + Send + Sync + 'static,
{
  fn request(&mut self, requested: usize) {
    let o_c = self.observer.clone();
    let h = self.scheduler.schedule_repeating(
      move |i| {
        o_c.write().unwrap().next(i as u128);
      },
      self.dur,
      self.at,
      Some(requested as usize),
    );
    if self.abort.is_some() {
      self.abort.clone().unwrap().unsubscribe();
    }
    self.abort = Some(h);
  }

  fn unsubscribe(&mut self) {
    if self.abort.is_some() {
      self.abort.clone().unwrap().unsubscribe();
    }
  }

  fn is_closed(&self) -> bool {
    if self.abort.is_some() {
      return self.abort.clone().unwrap().is_closed();
    }
    true // TODO: true if not started
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
    interval(Duration::from_millis(1), local.spawner())
      .take(5)
      .subscribe(move |_| (*ticks_c.lock().unwrap()) += 1);
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
