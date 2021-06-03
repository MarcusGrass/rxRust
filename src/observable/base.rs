use crate::prelude::*;
use crate::subscriber::Subscriber;

pub trait PublisherFactory {
  type Item: Send + 'static;
  type Err: Send + 'static;
}

pub trait LocalPublisherFactory<'a>: PublisherFactory {
  fn subscribe(self, channel: PublisherChannel<Self::Item, Self::Err>);
}

pub trait Source: SubscriptionLike {
  type Item: Send + 'static;
  type Err: Send + 'static;
}

pub trait SharedPublisherFactory: PublisherFactory {
  fn subscribe<S>(
    self,
    subscriber: S,
  )
  where
    S: Subscriber<Item = Self::Item, Err = Self::Err> + Send + Sync + 'static;
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
  fn actual_subscribe(self, channel: PublisherChannel<Self::Item, Self::Err>) {
    println!("{:?}", "sub");
    self.0.subscribe(channel)
  }
}

impl<Emit> SharedObservable for ObservableBase<Emit>
where
  Emit: SharedPublisherFactory,
{

  fn actual_subscribe(self, channel: PublisherChannel<Self::Item, Self::Err>) {
    //self.0.subscribe(channel)
  }
}
