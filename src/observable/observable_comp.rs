use crate::prelude::*;
use std::sync::{Mutex, Arc};

pub struct ObserverComp<N, C, Item> {
  next: N,
  complete: C,
  is_stopped: bool,
  upstream: Upstream<Item, ()>,
  marker: TypeHint<*const Item>,
}

impl<N, C, Item> Subscriber for ObserverComp<N, C, Item>
  where
      C: FnMut(),
      N: FnMut(Item),
{
  fn connect(&mut self, mut chn: SubscriptionChannel<Self::Item, Self::Err>) {
    chn.request(usize::MAX);
    self.upstream = Upstream::INIT(chn);
    //*self.upstream.lock().unwrap() = sub;
    //self.upstream.lock().unwrap().request(usize::MAX);
  }
}
impl<N, C, Item> SubscriptionLike for ObserverComp<N, C, Item>
  where
      C: FnMut(),
      N: FnMut(Item),
{
  fn request(&mut self, requested: usize) {
    println!("{:?}", "Intern req");
  }

  fn unsubscribe(&mut self) {
  }

  fn is_closed(&self) -> bool {
    todo!()
  }
}
impl<N, C, Item> Observer for ObserverComp<N, C, Item>
where
  C: FnMut(),
  N: FnMut(Item),
{
  type Item = Item;
  type Err = ();
  #[inline]
  fn next(&mut self, value: Item) {
    (self.next)(value);
  }
  #[inline]
  fn error(&mut self, _err: ()) { self.is_stopped = true; }
  fn complete(&mut self) {
    println!("{:?}", "Call complete");
    (self.complete)();
    self.is_stopped = true;
    self.upstream.unsubscribe();
  }
}

pub trait SubscribeComplete<'a, N, C> {
  /// A type implementing [`SubscriptionLike`]
  type Unsub: SubscriptionLike;

  /// Invokes an execution of an Observable and registers Observer handlers for
  /// notifications it will emit.
  fn subscribe_complete(self, next: N, complete: C) -> SubscriptionWrapper<Self::Unsub>;
}

impl<'a, S, N, C> SubscribeComplete<'a, N, C> for S
where
  S: LocalObservable<'a, Err = ()>,
  C: FnMut() + Send + Sync + 'static,
  N: FnMut(S::Item) + Send + Sync + 'static,
  S::Item: Send + 'static,
{
  type Unsub = LocalSubscription<'a>;
  fn subscribe_complete(self, next: N, complete: C) -> SubscriptionWrapper<Self::Unsub>
  where
    Self: Sized,
    S::Item: 'a,
  {
      println!("{:?}", "req");
    self.actual_subscribe(ObserverComp {
      next,
      complete,
      is_stopped: false,
      upstream: Upstream::UNINIT,
      marker: TypeHint::new(),
    });
    println!("{:?}", "req");
    SubscriptionWrapper(LocalSubscription::default())
  }
}

impl<'a, S, N, C> SubscribeComplete<'a, N, C> for Shared<S>
where
  S: SharedObservable<Err = ()>,
  C: FnMut() + Send + Sync + 'static,
  N: FnMut(S::Item) + Send + Sync + 'static,
  S::Item: 'static,
{
  type Unsub = SharedSubscription;
  fn subscribe_complete(self, next: N, complete: C) -> SubscriptionWrapper<Self::Unsub>
  where
    Self: Sized,
  {
    /*
    let mut unsub = self.0.actual_subscribe(Subscriber::shared(ObserverComp {
      next,
      complete,
      is_stopped: false,
      marker: TypeHint::new(),
    }));
    unsub.request(usize::MAX);
    SubscriptionWrapper(unsub)

     */
    SubscriptionWrapper(SharedSubscription::default())
  }
}

#[test]
fn raii() {
  /*
  let mut times = 0;
  {
    let mut subject = LocalSubject::new();
    {
      let _ = subject
        .clone()
        .subscribe_complete(|_| times += 1, || {})
        .unsubscribe_when_dropped();
    } // <-- guard is dropped here!
    subject.next(());
  }
  assert_eq!(times, 0);

   */

}

#[tokio::test(threaded_scheduler)]
async fn complete() {
  let cmpl = Arc::new(Mutex::new(false));
  let hits = Arc::new(Mutex::new(0));
  let obs = observable::from_iter(vec![1, 2, 3]);
  let cmpl_c = cmpl.clone();
  let hits_c = hits.clone();
  obs.subscribe_complete(move |n| *hits_c.lock().unwrap() += 1, move || *cmpl_c.lock().unwrap() = true);
  while !*cmpl.lock().unwrap() {

  }
  assert!(*cmpl.lock().unwrap());
  assert_eq!(*hits.lock().unwrap(), 3);

}
