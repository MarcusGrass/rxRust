use crate::prelude::*;
use crate::subscriber::Subscriber;

/// Shared wrap the Observable， subscribe and accept subscribe in a safe mode
/// by SharedObservable.
#[derive(Clone)]
pub struct Shared<R>(pub(crate) R);

pub trait SharedObservable: Observable {
  fn actual_subscribe<
    S: Subscriber<Item = Self::Item, Err = Self::Err> + Sync + Send + 'static,
  >(
    self,
    subscriber: S,
  );

  /// Convert to a thread-safe mode.
  #[inline]
  fn into_shared(self) -> Shared<Self>
  where
    Self: Sized,
  {
    Shared(self)
  }
}

observable_proxy_impl!(Shared, S);

impl<SO> SharedObservable for Shared<SO>
where
    SO: SharedObservable,
{

  fn actual_subscribe<
    S: Subscriber<Item=Self::Item, Err=Self::Err> + Sync + Send + 'static,
  >(self, subscriber: S) {
    self.0.actual_subscribe(subscriber)
  }
}
