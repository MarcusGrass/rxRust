use async_std::prelude::FutureExt as AsyncFutureExt;
use futures::future::{lazy, AbortHandle, FutureExt};
use std::future::Future;

use crate::prelude::{SubscriptionLike, PublisherChannel, Publisher, Source, Subscriber, SubscriptionChannel, SubscriberChannelItem, PublisherChannelItem};
use futures::{StreamExt, TryStreamExt, TryFutureExt, Stream};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use futures::channel::mpsc::{Sender, Receiver};
use async_std::future::IntoFuture;
use futures::stream::Next;
use std::task::{Context, Poll};
use std::pin::Pin;
use crate::observer::Observer;

pub fn task_future<T>(
  task: impl FnOnce(T) + 'static,
  state: T,
  delay: Option<Duration>,
) -> (impl Future<Output = ()>, SpawnHandle) {
  let fut = lazy(|_| task(state)).delay(delay.unwrap_or_default());
  let (fut, handle) = futures::future::abortable(fut);
  (fut.map(|_| ()), SpawnHandle::new(handle))
}

/// A Scheduler is an object to order task and schedule their execution.
pub trait SharedScheduler {
  fn spawn<Fut>(&self, future: Fut)
  where
    Fut: Future<Output = ()> + Send + 'static;

  fn schedule<T: Send + 'static>(
    &self,
    task: impl FnOnce(T) + Send + 'static,
    delay: Option<Duration>,
    state: T,
  ) -> SpawnHandle {
    let (f, handle) = task_future(task, state, delay);
    self.spawn(f);
    handle
  }

  fn schedule_repeating(
    &self,
    task: impl FnMut(usize) + Send + 'static,
    time_between: Duration,
    at: Option<Instant>,
    take: Option<usize>,
  ) -> SpawnHandle {
    let (f, handle) = repeating_future(task, time_between, at, take);
    self.spawn(f.map(|_| ()));
    handle
  }
}

pub trait LocalScheduler {
  fn spawn<Fut>(&self, future: Fut)
  where
    Fut: Future<Output = ()> + 'static;

  fn schedule<T: 'static>(
    &self,
    task: impl FnOnce(T) + 'static,
    delay: Option<Duration>,
    state: T,
  ) -> SpawnHandle {
    let (f, handle) = task_future(task, state, delay);
    self.spawn(f);
    handle
  }

  fn schedule_repeating(
    &self,
    task: impl FnMut(usize) + 'static,
    time_between: Duration,
    at: Option<Instant>,
    take: Option<usize>,
  ) -> SpawnHandle {
    let (f, handle) = repeating_future(task, time_between, at, take);
    self.spawn(f.map(|_| ()));
    handle
  }
}

#[derive(Clone)]
pub struct SpawnHandle {
  pub handle: AbortHandle,
  is_closed: Arc<RwLock<bool>>,
}

impl SpawnHandle {
  #[inline]
  pub fn new(handle: AbortHandle) -> Self {
    SpawnHandle {
      handle,
      is_closed: Arc::new(RwLock::new(false)),
    }
  }
}

impl SubscriptionLike for SpawnHandle {
  fn request(&mut self, _: usize) {}

  fn unsubscribe(&mut self) {
    let was_closed = *self.is_closed.read().unwrap();
    if !was_closed {
      *self.is_closed.write().unwrap() = true;
      self.handle.abort();
    }
  }

  #[inline]
  fn is_closed(&self) -> bool { *self.is_closed.read().unwrap() }
}

#[cfg(feature = "futures-scheduler")]
mod futures_scheduler {
  use crate::scheduler::{LocalScheduler, SharedScheduler};
  use futures::{
    executor::{LocalSpawner, ThreadPool},
    task::{LocalSpawnExt, SpawnExt},
    Future, FutureExt,
  };

  impl SharedScheduler for ThreadPool {
    fn spawn<Fut>(&self, future: Fut)
    where
      Fut: Future<Output = ()> + Send + 'static,
    {
      SpawnExt::spawn(self, future).unwrap();
    }
  }

  impl LocalScheduler for LocalSpawner {
    fn spawn<Fut>(&self, future: Fut)
    where
      Fut: Future<Output = ()> + 'static,
    {
      self.spawn_local(future.map(|_| ())).unwrap();
    }
  }
}

fn repeating_future(
  task: impl FnMut(usize) + 'static,
  time_between: Duration,
  at: Option<Instant>,
  take: Option<usize>,
) -> (impl Future<Output = ()>, SpawnHandle) {
  let now = Instant::now();
  let delay = at.map(|inst| {
    if inst > now {
      inst - now
    } else {
      Duration::from_micros(0)
    }
  });
  let future = to_interval(
    task,
    time_between,
    delay.unwrap_or(time_between),
    take.unwrap_or(usize::MAX),
  );
  let (fut, handle) = futures::future::abortable(future);
  (fut.map(|_| ()), SpawnHandle::new(handle))
}

fn to_interval(
  mut task: impl FnMut(usize) + 'static,
  interval_duration: Duration,
  delay: Duration,
  take: usize,
) -> impl Future<Output = ()> {
  let mut number = 0;

  futures::future::ready(())
    .then(move |_| {
      task(number);
      async_std::stream::interval(interval_duration)
        .take(take)
        .for_each(move |_| {
          number += 1;
          task(number);
          futures::future::ready(())
        })
    })
    .delay(delay)
}

pub fn observe_op<S: Subscriber + Send + 'static>(subscriber: S, inc: Receiver<SubscriberChannelItem<S::Item, S::Err>>, req: Receiver<PublisherChannelItem>) {
  tokio::spawn(do_observe_op(subscriber, inc, req));
}

async fn do_observe_op<S: Subscriber + Send + 'static>(mut subscriber: S, mut inc: Receiver<SubscriberChannelItem<S::Item, S::Err>>, mut req: Receiver<PublisherChannelItem>) {
  loop {
    let option = req.next().await;
    if let Some(requested) = option {
      match requested {
        PublisherChannelItem::Request(r) => {
          subscriber.request(r);
          let mut countdown = r;
          while countdown > 0 {
            println!("{:?}", "do op");
            let incoming_item = inc.next().await;
            println!("{:?}", "inc next");
            if let Some(item) = incoming_item {
              match item {
                SubscriberChannelItem::Item(i) => {
                  subscriber.next(i);
                  countdown -= 1;
                }
                SubscriberChannelItem::Err(e) => {
                  subscriber.error(e);
                  break;
                }
                SubscriberChannelItem::Complete => {
                  subscriber.complete();
                  break;
                }
              }
            } else {
              break;
            }
          }
          break;
        }
        PublisherChannelItem::Cancel => {
          subscriber.unsubscribe();
          break;
        }
      }
    } else {
      break;
    }
  }
}

pub fn observe<S: Subscriber + Send + 'static>(mut subscriber: S, mut rec: Receiver<SubscriberChannelItem<S::Item, S::Err>>) {
    println!("{:?}", "do observe");
  tokio::spawn(do_observe(subscriber, rec));
}
async fn do_observe<S: Subscriber + Send + 'static>(mut subscriber: S, mut rec: Receiver<SubscriberChannelItem<S::Item, S::Err>>) {
  loop {
    let option = rec.next().await;
    if let Some(item) = option {
      match item {
        SubscriberChannelItem::Item(v) => subscriber.next(v),
        SubscriberChannelItem::Err(e) => {
          subscriber.error(e);
          break;
        }
        SubscriberChannelItem::Complete => {
          subscriber.complete();
          break;
        }
      }
    } else {
      break;
    }
  }

}

pub fn publish<P: Source + Send + 'static>(mut publisher: P, rec: Receiver<PublisherChannelItem>) {
  tokio::spawn(do_publish(publisher, rec));
}


async fn do_publish<P: Source + Send + 'static>(mut publisher: P, mut rec: Receiver<PublisherChannelItem>) {
  loop {
    println!("{:?}", "Await pub item");
    let option = rec.next().await;
    println!("Inc pub item");
    if let Some(item) = option {
      match item {
        PublisherChannelItem::Request(r) => publisher.request(r),
        PublisherChannelItem::Cancel => {
          publisher.unsubscribe();
          break;
        }
      }
    } else {
      break;
    }
  }

}

async fn await_cancel(mut unsub: Receiver<()>) -> bool {
  unsub.next().await
      .map(|_| true)
      .unwrap_or(false)
}

async fn publish_on_next<P: Source + Send + 'static>(mut request: Receiver<usize>, mut publisher: P) {
  loop {
    let req = request.next().await.unwrap();
    publisher.request(req);
  }
}



mod tokio_scheduler {
  use super::*;
  use std::sync::Arc;
  use tokio::runtime::Runtime;
  use crate::prelude::{Publisher, PublisherChannel};

  impl SharedScheduler for Runtime {
    fn spawn<Fut>(&self, future: Fut)
    where
      Fut: Future<Output = ()> + Send + 'static,
    {
      Runtime::spawn(self, future);
    }
  }

  impl SharedScheduler for Arc<Runtime> {
    fn spawn<Fut>(&self, future: Fut)
    where
      Fut: Future<Output = ()> + Send + 'static,
    {
      Runtime::spawn(self, future);
    }
  }


}

#[cfg(all(test, feature = "tokio-scheduler"))]
mod test {
  use crate::prelude::*;
  use bencher::Bencher;
  use futures::executor::{LocalPool, ThreadPool};
  use std::sync::{Arc, Mutex};

  fn waste_time(v: u32) -> u32 {
    (0..v)
      .into_iter()
      .map(|index| (0..index).sum::<u32>().min(u32::MAX / v))
      .sum()
  }

  #[test]
  fn bench_pool() { do_bench_pool(); }

  benchmark_group!(do_bench_pool, pool);

  fn pool(b: &mut Bencher) {
    let last = Arc::new(Mutex::new(0));
    b.iter(|| {
      let c_last = last.clone();
      let pool = ThreadPool::new().unwrap();
      observable::from_iter(0..1000)
        .observe_on(pool)
        .map(waste_time)
        .into_shared()
        .subscribe(move |v| *c_last.lock().unwrap() = v);

      // todo: no way to wait all task has finished in `ThreadPool`.

      *last.lock().unwrap()
    })
  }

  #[test]
  fn bench_local_thread() { do_bench_local_thread(); }

  benchmark_group!(do_bench_local_thread, local_thread);

  fn local_thread(b: &mut Bencher) {
    let last = Arc::new(Mutex::new(0));
    b.iter(|| {
      let c_last = last.clone();
      let mut local = LocalPool::new();
      observable::from_iter(0..1000)
        .observe_on(local.spawner())
        .map(waste_time)
        .subscribe(move |v| *c_last.lock().unwrap() = v);
      local.run();
      *last.lock().unwrap()
    })
  }

  #[test]
  fn bench_tokio_basic() { do_bench_tokio_basic(); }

  benchmark_group!(do_bench_tokio_basic, tokio_basic);

  fn tokio_basic(b: &mut Bencher) {
    use tokio::runtime;
    let last = Arc::new(Mutex::new(0));
    b.iter(|| {
      let c_last = last.clone();
      let local = runtime::Builder::new().basic_scheduler().build().unwrap();

      observable::from_iter(0..1000)
        .observe_on(local)
        .map(waste_time)
        .into_shared()
        .subscribe(move |v| *c_last.lock().unwrap() = v);

      // todo: no way to wait all task has finished in `Tokio` Scheduler.
      *last.lock().unwrap()
    })
  }

  #[test]
  fn bench_tokio_thread() { do_bench_tokio_thread(); }

  benchmark_group!(do_bench_tokio_thread, tokio_thread);

  fn tokio_thread(b: &mut Bencher) {
    use tokio::runtime;
    let last = Arc::new(Mutex::new(0));
    b.iter(|| {
      let c_last = last.clone();
      let pool = runtime::Builder::new()
        .threaded_scheduler()
        .build()
        .unwrap();
      observable::from_iter(0..1000)
        .observe_on(pool)
        .map(waste_time)
        .into_shared()
        .subscribe(move |v| *c_last.lock().unwrap() = v);

      // todo: no way to wait all task has finished in `Tokio` Scheduler.

      *last.lock().unwrap()
    })
  }
}
