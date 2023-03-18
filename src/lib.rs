use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Vfhm<'lut, T, const SIZE: usize> {
  lut: &'lut Lut<'lut>,
  inner: [Option<T>; SIZE],
}

impl<'lut, T, const SIZE: usize> Vfhm<'lut, T, SIZE> {
  pub fn new(lut: &'lut Lut<'lut>) -> Self {
    Vfhm {
      lut,
      inner: [(); SIZE].map(|_| Option::<T>::default()),
    }
  }

  pub fn get<K: VfhmKey>(&self, key: K) -> Option<&T> {
    let index = key.key_index(self.lut);

    if index < SIZE && key.is_same_key(self.lut.key(index)) {
      self.inner[index].as_ref()
    } else {
      None
    }
  }

  pub fn insert<K: VfhmKey>(&mut self, key: K, value: T) -> Option<T> {
    let mut output = Some(value);

    std::mem::swap(&mut output, &mut self.inner[key.key_index(self.lut)]);

    output
  }

  pub fn remove<K: VfhmKey>(&mut self, key: K) -> Option<T> {
    let mut output = None;

    std::mem::swap(&mut output, &mut self.inner[key.key_index(self.lut)]);

    output
  }
}

pub trait VfhmKey {
  #[inline]
  fn char_index(val: char) -> usize {
    val as usize - 32
  }

  fn key_index(&self, lut: &[usize]) -> usize;

  fn is_same_key(&self, key: &str) -> bool;
}

impl<T> VfhmKey for T
where
  T: AsRef<str>,
{
  fn key_index(&self, lut: &[usize]) -> usize {
    self
      .as_ref()
      .chars()
      .map(Self::char_index)
      .fold(0, |agg, index| agg + lut[index])
  }

  /// TODO: check speed
  fn is_same_key(&self, key: &str) -> bool {
    self
      .as_ref()
      .as_bytes()
      .iter()
      .zip(key.as_bytes())
      .all(|(a, b)| a == b)
  }
}

pub struct LutBuilder<'lut>(pub Vec<&'lut str>);

impl<'lut> LutBuilder<'lut> {
  pub const LUT_SIZE: usize = 96;

  pub fn build(self) -> Lut<'lut> {
    let LutBuilder(keys) = self;
    let inner = [0; LutBuilder::LUT_SIZE];

    Lut { keys, inner }
  }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Lut<'lut> {
  keys: Vec<&'lut str>,
  inner: [usize; LutBuilder::LUT_SIZE],
}

impl Lut<'_> {
  pub fn key(&self, index: usize) -> &str {
    self.keys[index]
  }
}

impl Deref for Lut<'_> {
  type Target = [usize];

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl DerefMut for Lut<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}
