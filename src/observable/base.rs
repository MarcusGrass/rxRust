use crate::prelude::*;

pub trait PublisherFactory {
  type Item;
  type Err;
}

pub trait LocalPublisherFactory<'a>: PublisherFactory {
  fn subscribe<O>(
    self,
    subscriber: Subscriber<O, LocalSubscription<'a>>,
  ) -> LocalSubscription<'a>
  where
    O: Observer<Item = Self::Item, Err = Self::Err> + 'a;
}

pub trait SharedPublisherFactory: PublisherFactory {
  fn subscribe<O>(
    self,
    subscriber: Subscriber<O, SharedSubscription>,
  ) -> SharedSubscription
  where
    O: Observer<Item = Self::Item, Err = Self::Err> + Send + Sync + 'static;
}

#[derive(Clone)]
pub struct ObservableBase<Emit>(Emit);

impl<Emit> ObservableBase<Emit> {
  pub fn new(emitter: Emit) -> Self { ObservableBase(emitter) }
}

impl<Emit> Observable for ObservableBase<Emit>
where
  Emit: PublisherFactory,
{
  type Item = Emit::Item;
  type Err = Emit::Err;
}

impl<'a, Emit> LocalObservable<'a> for ObservableBase<Emit>
where
  Emit: LocalPublisherFactory<'a>,
{
  type Unsub = LocalSubscription<'a>;
  fn actual_subscribe<O>(
    self,
    subscriber: Subscriber<O, LocalSubscription<'a>>,
  ) -> Self::Unsub
  where
    O: Observer<Item = Self::Item, Err = Self::Err> + 'a,
  {
    self.0.subscribe(subscriber)
  }
}

impl<Emit> SharedObservable for ObservableBase<Emit>
where
  Emit: SharedPublisherFactory,
{
  type Unsub = SharedSubscription;
  fn actual_subscribe<O>(
    self,
    subscriber: Subscriber<O, SharedSubscription>,
  ) -> Self::Unsub
  where
    O: Observer<Item = Self::Item, Err = Self::Err> + Send + Sync + 'static,
  {
    self.0.subscribe(subscriber)
  }
}
