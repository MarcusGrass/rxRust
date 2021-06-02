use crate::prelude::*;
use std::sync::{Mutex, Arc};

#[derive(Clone)]
pub struct ObserverComp<N, C, Item> {
  next: N,
  complete: C,
  is_stopped: bool,
  marker: TypeHint<*const Item>,
}
impl<N, C, Item> Subscriber for ObserverComp<N, C, Item>
  where
      C: FnMut(),
      N: FnMut(Item),
{
  fn connect(&self, mut chn: SubscriptionChannel<Self::Item, Self::Err>) {
    chn.request(usize::MAX);
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
    (self.complete)();
    self.is_stopped = true;
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
  C: FnMut() + Send + 'static,
  N: FnMut(S::Item) + Send + 'static,
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
  let obs = observable::from_iter(vec![1, 2, 3]);
  obs.subscribe_complete(|n| println!("{:?}", n), || println!("{:?}", "do"));
}
