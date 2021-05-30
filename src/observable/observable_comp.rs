use crate::prelude::*;

#[derive(Clone)]
pub struct ObserverComp<N, S, C, Item> {
  next: N,
  complete: C,
  is_stopped: bool,
  upstream: S,
  marker: TypeHint<*const Item>,
}

impl<N, S, C, Item> Subscriber<S> for ObserverComp<N, S, C, Item>
  where
      C: FnMut(),
      N: FnMut(Item),
      S: SubscriptionLike
{
  fn on_subscribe(&self, sub: S) {
    todo!()
  }
}

impl<N, S, C, Item> Observer for ObserverComp<N, S, C, Item>
where
  C: FnMut(),
  N: FnMut(Item),
  S: SubscriptionLike
{
  type Item = Item;
  type Err = ();
  #[inline]
  fn next(&mut self, value: Item) { (self.next)(value); }
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
  C: FnMut() + 'a,
  N: FnMut(S::Item) + 'a,
  S::Item: 'a,
{
  type Unsub = LocalSubscription<'a>;
  fn subscribe_complete(self, next: N, complete: C) -> SubscriptionWrapper<Self::Unsub>
  where
    Self: Sized,
    S::Item: 'a,
  {
    /*
    let mut unsub = self.actual_subscribe(Subscriber::local(ObserverComp {
      next,
      complete,
      is_stopped: false,
      marker: TypeHint::new(),
    }));
    unsub.request(usize::MAX);
    SubscriptionWrapper(unsub)

     */
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
}
