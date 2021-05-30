use crate::prelude::*;
use crate::subscriber::Subscriber;

pub trait PublisherFactory {
  type Item;
  type Err;
}

pub trait LocalPublisherFactory<'a>: PublisherFactory {
  fn subscribe<S>(
    self,
    subscriber: S,
  )
  where
    S: Subscriber<LocalSubscription<'a>, Item = Self::Item, Err = Self::Err> + 'a;
}

pub trait SharedPublisherFactory: PublisherFactory {
  fn subscribe<S>(
    self,
    subscriber: S,
  )
  where
    S: Subscriber<SharedSubscription, Item = Self::Item, Err = Self::Err> + Send + Sync + 'static;
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
  fn actual_subscribe<Sub: Subscriber<LocalSubscription<'a>, Item=Self::Item, Err=Self::Err> + 'a>(self, subscriber: Sub) {
    self.0.subscribe(subscriber)
  }
}

impl<Emit> SharedObservable for ObservableBase<Emit>
where
  Emit: SharedPublisherFactory,
{

  fn actual_subscribe<
    S: Subscriber<SharedSubscription, Item=Self::Item, Err=Self::Err> + Sync + Send + 'static,
  >(self, subscriber: S) {
    self.0.subscribe(subscriber)
  }
}
