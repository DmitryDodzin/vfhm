use std::{
  marker::PhantomData,
  ops::{Deref, DerefMut},
};

use crate::{Vfhm, VfhmParams};

pub trait VfhmStaticMap {
  const SEED: usize;
  const MASK: usize;
  const MASK_OFFSET: usize;
  const BONDS: (usize, usize);

  fn into_params() -> VfhmParams {
    VfhmParams(Self::SEED, Self::MASK, Self::MASK_OFFSET, Self::BONDS)
  }
}

pub struct StaticVfhm<K, V, S>(Vfhm<K, V>, PhantomData<S>);

impl<K, V, S> StaticVfhm<K, V, S>
where
  S: VfhmStaticMap,
{
  pub fn new() -> Self {
    StaticVfhm(Vfhm::with_params(S::into_params()), PhantomData::<S>)
  }
}

impl<K, V, S> Default for StaticVfhm<K, V, S>
where
  S: VfhmStaticMap,
{
  fn default() -> Self {
    Self::new()
  }
}

impl<K, V, S> Deref for StaticVfhm<K, V, S> {
  type Target = Vfhm<K, V>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<K, V, S> DerefMut for StaticVfhm<K, V, S> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T> From<T> for VfhmParams
where
  T: VfhmStaticMap,
{
  fn from(_: T) -> Self {
    T::into_params()
  }
}
