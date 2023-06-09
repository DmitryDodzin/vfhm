#![allow(dead_code)]
#![allow(unused_variables)]

use std::{borrow::Borrow, fmt, marker::PhantomData, mem};

pub trait VfhmConstParams {
  const SEED: usize;
  const MASK: usize;
  const MASK_OFFSET: usize;
}

#[derive(Debug, Clone, Copy)]
pub struct VfhmParams(pub usize, pub usize, pub usize);

impl<T> From<T> for VfhmParams
where
  T: VfhmConstParams,
{
  fn from(consts: T) -> Self {
    VfhmParams(T::SEED, T::MASK, T::MASK_OFFSET)
  }
}

#[derive(Debug, Clone)]
pub struct Vfhm<K, V> {
  table: Vec<Option<(K, V)>>,
  params: VfhmParams,
}

impl<K, V> Vfhm<K, V> {
  pub fn new<P>(params: P) -> Self
  where
    P: Into<VfhmParams>,
  {
    let params = params.into();
    let lenght = (params.1 >> params.2) + 1;

    Vfhm {
      table: (0..lenght).map(|_| None).collect(),
      params,
    }
  }

  pub fn builder() -> VfhmBuilder<K, V> {
    VfhmBuilder::default()
  }
}

impl<K, V> Vfhm<K, V>
where
  K: VfhmKey,
{
  pub fn get<Q>(&self, key: Q) -> Option<&V>
  where
    Q: Borrow<K>,
  {
    let Vfhm { ref table, params } = *self;
    let key = key.borrow();

    let index = key.table_key(params);

    table[index]
      .iter()
      .find(|(k, _)| key.table_key_compare(k))
      .map(|(_, value)| value)
  }

  pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)>
  where
    V: fmt::Debug,
  {
    let Vfhm {
      ref mut table,
      params,
    } = *self;

    let index = key.table_key(params);

    let mut output = Some((key, value));

    std::mem::swap(&mut output, &mut table[index]);

    output
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
    !self
      .as_ref()
      .iter()
      .zip(other.as_ref().iter())
      .any(|(a, b)| a != b)
  }
}

#[derive(Debug)]
pub struct VfhmBuilder<K, V> {
  keys: Vec<K>,
  params: VfhmParams,
  _values: PhantomData<V>,
}

impl<K, V> VfhmBuilder<K, V>
where
  K: AsRef<[u8]> + fmt::Debug,
{
  pub fn set_keys(&mut self, keys: Vec<K>) -> &mut Self {
    self.keys = keys;

    self
  }

  pub fn find_params(&mut self, max_iterations: usize) -> &mut Self {
    for iteration in 0..max_iterations {
      let mut map = Vfhm::new(self.params);

      for (index, key) in self.keys.iter().enumerate() {
        map.insert(key, index);
      }

      if self
        .keys
        .iter()
        .enumerate()
        .all(|(index, key)| map.get(key) == Some(&index))
      {
        break;
      }

      let VfhmParams(mut seed, mut mask, mut mask_offset) = self.params;

      if seed == mask >> mask_offset {
        seed = 0;

        if mask_offset + ((1.0 + (mask >> mask_offset) as f64).sqrt() as usize) + 1
          < mem::size_of::<usize>() * 8
        {
          mask <<= 1;
          mask_offset += 1;
        } else {
          mask = (mask >> (mask_offset - 1)) | 1;
          mask_offset = 0;
        }
      } else {
        seed += 1;
      }

      self.params = VfhmParams(seed, mask, mask_offset);
    }

    self
  }

  pub fn build(&self) -> Vfhm<K, V> {
    Vfhm::new(self.params)
  }
}

impl<K, V> Default for VfhmBuilder<K, V> {
  fn default() -> Self {
    VfhmBuilder {
      keys: Vec::new(),
      params: VfhmParams(0, 0b11, 0),
      _values: PhantomData::<V>,
    }
  }
}

impl<K, V> AsRef<Vec<K>> for VfhmBuilder<K, V> {
  fn as_ref(&self) -> &Vec<K> {
    &self.keys
  }
}

impl<K, V> AsMut<Vec<K>> for VfhmBuilder<K, V> {
  fn as_mut(&mut self) -> &mut Vec<K> {
    &mut self.keys
  }
}

#[cfg(test)]
mod tests {

  struct DaysParams;

  impl VfhmConstParams for DaysParams {
    const SEED: usize = 1;
    const MASK: usize = 112;
    const MASK_OFFSET: usize = 4;
  }

  use super::*;

  #[test]
  fn builder() {
    let mut hashmap = Vfhm::builder()
      .set_keys(vec![
        "sunday",
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "firday",
        "saturday",
      ])
      .find_params(usize::MAX)
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
    let mut hashmap = Vfhm::new(DaysParams);

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
