use std::{borrow::Borrow, marker::PhantomData};

use crate::params::{VfhmConstParams, VfhmDefaultParams, VfhmParams};

pub mod builder;
pub mod params;

#[derive(Debug, Clone)]
pub struct Vfhm<K, V, CP = VfhmDefaultParams> {
  table: Vec<Option<(K, V)>>,
  params: VfhmParams,
  _params_maker: PhantomData<CP>,
}

impl<K, V, CP> Vfhm<K, V, CP> {
  pub fn new() -> Self
  where
    CP: VfhmConstParams,
  {
    Self::with_params(CP::into_params())
  }

  pub fn with_params<P>(maybe_params: P) -> Self
  where
    P: Into<VfhmParams>,
  {
    let params = maybe_params.into();

    Vfhm {
      table: (0..params.mask_size()).map(|_| None).collect(),
      params,
      _params_maker: PhantomData::<CP>,
    }
  }
}

impl<K, V, CP> Vfhm<K, V, CP>
where
  K: VfhmKey,
{
  pub fn get<Q>(&self, key: Q) -> Option<&V>
  where
    Q: Borrow<K>,
  {
    let Vfhm {
      ref table, params, ..
    } = *self;
    let key = key.borrow();

    let index = key.table_key(params);

    table[index]
      .iter()
      .find(|(k, _)| key.table_key_compare(k))
      .map(|(_, value)| value)
  }

  pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
    let Vfhm {
      ref mut table,
      params,
      ..
    } = *self;

    let index = key.table_key(params);

    let mut output = Some((key, value));

    std::mem::swap(&mut output, &mut table[index]);

    output
  }
}

impl<K, V, CP> Default for Vfhm<K, V, CP>
where
  CP: VfhmConstParams,
{
  fn default() -> Self {
    Self::new()
  }
}

pub trait VfhmKey {
  fn table_key(&self, params: VfhmParams) -> usize;

  fn table_key_compare(&self, other: &Self) -> bool;
}

impl<T> VfhmKey for T
where
  T: AsRef<[u8]>,
{
  fn table_key(&self, VfhmParams(seed, mask, mask_offset): VfhmParams) -> usize {
    let mut index: usize = 1;

    for byte in self.as_ref() {
      index = index.wrapping_mul(*byte as usize).wrapping_sub(seed)
    }

    (index & mask) >> mask_offset
  }

  fn table_key_compare(&self, other: &Self) -> bool {
    self.as_ref() == other.as_ref()
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::{builder::VfhmBuilder, params::VfhmConstParams};

  struct DaysParams;

  impl VfhmConstParams for DaysParams {
    const SEED: usize = 1;
    const MASK: usize = 112;
    const MASK_OFFSET: usize = 4;
  }

  #[test]
  fn builder() {
    let mut hashmap = VfhmBuilder::default()
      .set_keys(vec![
        "sunday",
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "firday",
        "saturday",
      ])
      .find_params(1000)
      .build();

    hashmap.insert("sunday", 1);
    hashmap.insert("monday", 2);
    hashmap.insert("tuesday", 3);
    hashmap.insert("wednesday", 4);
    hashmap.insert("thursday", 5);
    hashmap.insert("firday", 6);
    hashmap.insert("saturday", 7);

    assert_eq!(hashmap.get("sunday"), Some(&1));
    assert_eq!(hashmap.get("monday"), Some(&2));
    assert_eq!(hashmap.get("tuesday"), Some(&3));
    assert_eq!(hashmap.get("wednesday"), Some(&4));
    assert_eq!(hashmap.get("thursday"), Some(&5));
    assert_eq!(hashmap.get("firday"), Some(&6));
    assert_eq!(hashmap.get("saturday"), Some(&7));
  }

  #[test]
  fn consts() {
    let mut hashmap = Vfhm::<_, _, DaysParams>::new();

    hashmap.insert("sunday", 1);
    hashmap.insert("monday", 2);
    hashmap.insert("tuesday", 3);
    hashmap.insert("wednesday", 4);
    hashmap.insert("thursday", 5);
    hashmap.insert("firday", 6);
    hashmap.insert("saturday", 7);

    assert_eq!(hashmap.get("sunday"), Some(&1));
    assert_eq!(hashmap.get("monday"), Some(&2));
    assert_eq!(hashmap.get("tuesday"), Some(&3));
    assert_eq!(hashmap.get("wednesday"), Some(&4));
    assert_eq!(hashmap.get("thursday"), Some(&5));
    assert_eq!(hashmap.get("firday"), Some(&6));
    assert_eq!(hashmap.get("saturday"), Some(&7));
  }
}
