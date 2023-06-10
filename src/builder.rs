use std::{marker::PhantomData, mem};

use crate::{
  params::{VfhmDefaultParams, VfhmParams},
  Vfhm,
};

#[derive(Debug)]
pub struct VfhmBuilder<K, V> {
  keys: Vec<K>,
  params: VfhmParams,
  _values: PhantomData<V>,
}

impl<K, V> VfhmBuilder<K, V>
where
  K: AsRef<[u8]>,
{
  pub fn set_keys(&mut self, keys: Vec<K>) -> &mut Self {
    self.keys = keys;

    self
  }

  pub fn find_params(&mut self, max_iterations: usize) -> &mut Self {
    for _ in 0..max_iterations {
      let mut map = Vfhm::<_, _, VfhmDefaultParams>::with_params(self.params);

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
    Vfhm::with_params(self.params)
  }
}

impl<K, V> Default for VfhmBuilder<K, V> {
  fn default() -> Self {
    VfhmBuilder {
      keys: Vec::new(),
      params: VfhmParams(0, 1, 0),
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
