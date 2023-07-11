use std::borrow::Borrow;

#[cfg(feature = "builder")]
pub mod builder;
pub mod r#static;

#[derive(Debug, Clone)]
pub struct Vfhm<K, V> {
  table: Vec<Option<(K, V)>>,
  params: VfhmParams,
  length: usize,
}

impl<K, V> Vfhm<K, V> {
  pub fn with_params<P>(maybe_params: P) -> Self
  where
    P: Into<VfhmParams>,
  {
    let params = maybe_params.into();

    Vfhm {
      table: (0..params.mask_size()).map(|_| None).collect(),
      params,
      length: 0,
    }
  }
}

impl<K, V> Vfhm<K, V>
where
  K: VfhmKey,
{
  pub fn len(&self) -> usize {
    self.length
  }

  pub fn is_empty(&self) -> bool {
    self.length == 0
  }

  pub fn contains_key<Q>(&self, key: Q) -> bool
  where
    Q: Borrow<K>,
  {
    self.params.bound_check(key.borrow()) && self.get(key).is_some()
  }

  pub fn get<Q>(&self, key: Q) -> Option<&V>
  where
    Q: Borrow<K>,
  {
    let Vfhm {
      ref table, params, ..
    } = *self;
    let key = key.borrow();

    if !self.params.bound_check(key) {
      return None;
    }

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

    if output.is_none() {
      self.length += 1;
    }

    output
  }

  pub fn remove<Q>(&mut self, key: K) -> Option<(K, V)>
  where
    Q: Borrow<K>,
  {
    let Vfhm {
      ref mut table,
      params,
      ..
    } = *self;

    let index = key.table_key(params);

    let mut output = None;

    std::mem::swap(&mut output, &mut table[index]);

    if output.is_some() {
      self.length -= 1;
    }

    output
  }
}

#[derive(Debug, Clone, Copy)]
pub struct VfhmParams(pub usize, pub usize, pub usize, pub (usize, usize));

impl VfhmParams {
  pub fn mask_size(&self) -> usize {
    let VfhmParams(_, mask, mask_offset, _) = *self;
    (mask >> mask_offset) + 1
  }

  pub fn bounds_mut(&mut self) -> &mut (usize, usize) {
    &mut self.3
  }

  pub fn bound_check<K>(&self, key: &K) -> bool
  where
    K: VfhmKey,
  {
    let (lower, upper) = self.3;
    let len = key.key_len();

    lower <= len && len <= upper
  }
}

pub trait VfhmKey {
  fn key_len(&self) -> usize;

  fn table_key(&self, params: VfhmParams) -> usize;

  fn table_key_compare(&self, other: &Self) -> bool;
}

impl<T> VfhmKey for T
where
  T: AsRef<[u8]>,
{
  #[inline]
  fn key_len(&self) -> usize {
    self.as_ref().len()
  }

  fn table_key(&self, VfhmParams(seed, mask, mask_offset, _): VfhmParams) -> usize {
    let mut index: usize = 1;

    for byte in self.as_ref() {
      index = index.wrapping_mul(*byte as usize).wrapping_sub(seed)
    }

    (index & mask) >> mask_offset
  }

  #[inline]
  fn table_key_compare(&self, other: &Self) -> bool {
    self.as_ref() == other.as_ref()
  }
}

#[cfg(test)]
mod tests {

  use crate::{
    builder::VfhmBuilder,
    r#static::{StaticVfhm, VfhmStaticMap},
  };

  struct DaysParams;

  impl VfhmStaticMap for DaysParams {
    const SEED: usize = 1;
    const MASK: usize = 112;
    const MASK_OFFSET: usize = 4;
    const BONDS: (usize, usize) = (6, 9);
  }

  type DaysMap<K, V> = StaticVfhm<K, V, DaysParams>;

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
    let mut hashmap = DaysMap::new();

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
