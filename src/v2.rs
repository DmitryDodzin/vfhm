use std::{
  borrow::Borrow,
  collections::HashSet,
  hash::{Hash, Hasher},
};

const KEY_MASK: u64 = 0xff0;

#[derive(Debug, Clone)]
pub struct Vfhm<K: VfhmKey, V> {
  inner: Vec<Option<(K, V)>>,
  seed: u64,
  key_bounds: (usize, usize),
}

impl<K, V> Vfhm<K, V>
where
  K: VfhmKey,
{
  pub fn new(seed: u64, key_bounds: (usize, usize)) -> Self {
    Vfhm {
      inner: [(); KEY_MASK as usize].iter().map(|_| None).collect(),
      key_bounds,
      seed,
    }
  }

  pub fn get<Q>(&self, key: Q) -> Option<&V>
  where
    K: Borrow<Q>,
    Q: VfhmKey + PartialEq<K>,
  {
    let Vfhm {
      inner,
      seed,
      key_bounds,
    } = &self;

    if !key.check_bounds(key_bounds) {
      return None;
    }

    let index = key.key_hash(*seed);

    inner
      .get(index)
      .and_then(|bucket| bucket.iter().find(|(k, _)| &key == k))
      .map(|(_, value)| value)
  }

  pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)>
  where
    K: VfhmKey,
  {
    let Vfhm {
      ref mut inner,
      seed,
      ..
    } = self;

    let index = key.key_hash(*seed);

    let mut output = Some((key, value));

    std::mem::swap(&mut output, &mut inner[index]);

    output
  }
}

pub fn find_seed<K: VfhmKey>(keys: &[K]) -> u64 {
  for seed in 1..100000 {
    let hashes = keys
      .iter()
      .map(|day| day.key_hash(seed))
      .collect::<HashSet<_>>();

    if hashes.len() == keys.len() {
      return seed;
    }
  }

  panic!("After 100000 iterations unable to find seeds");
}

pub trait VfhmKey: PartialEq<Self> {
  fn key_hash(&self, seed: u64) -> usize;

  #[inline]
  fn check_bounds(&self, _: &(usize, usize)) -> bool {
    true
  }
}

impl<T> VfhmKey for T
where
  T: AsRef<str> + PartialEq,
{
  #[inline]
  fn key_hash(&self, seed: u64) -> usize {
    let mut hasher = SeedHasher::new(seed);

    self.as_ref().hash(&mut hasher);

    (hasher.finish() & KEY_MASK) as usize
  }

  #[inline]
  fn check_bounds(&self, (lower, upper): &(usize, usize)) -> bool {
    let key_len = self.as_ref().len();

    *lower <= key_len && key_len <= *upper
  }
}

#[derive(Default)]
pub struct SeedHasher(u64, u64);

impl SeedHasher {
  pub fn new(seed: u64) -> Self {
    SeedHasher(1, seed)
  }
}

impl Hasher for SeedHasher {
  #[inline]
  fn finish(&self) -> u64 {
    self.0
  }

  #[inline]
  fn write(&mut self, bytes: &[u8]) {
    let SeedHasher(mut hash, seed) = *self;

    for byte in bytes {
      hash = hash.wrapping_mul((*byte as u64) * seed)
    }

    *self = SeedHasher(hash, seed);
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  const DAYS: [&str; 7] = [
    "sunday",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "firday",
    "saturday",
  ];

  #[test]
  fn seed_hash() {
    let mut hashmap = Vfhm::new(find_seed(&DAYS), (6, 9));

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
