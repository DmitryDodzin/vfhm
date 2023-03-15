#![feature(test)]
#![feature(once_cell)]

use std::{
  hash::Hasher,
  ops::{Deref, DerefMut},
};

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
    let inner = [0; LutBuilder::LUT_SIZE];

    Lut {
      keys: self.0,
      inner,
    }
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

#[derive(Debug)]
pub struct VfhmHasher<'lut> {
  lut: &'lut Lut<'lut>,
  inner: u64,
}

impl<'lut> VfhmHasher<'lut> {
  pub fn new(lut: &'lut Lut<'lut>) -> Self {
    VfhmHasher { lut, inner: 0 }
  }
}

impl Hasher for VfhmHasher<'_> {
  #[inline]
  fn finish(&self) -> u64 {
    self.inner
  }

  #[inline]
  fn write(&mut self, bytes: &[u8]) {
    bytes
      .iter()
      .copied()
      .map(char::from)
      .map(|byte| byte as usize - 32)
      .filter(|byte| byte < &LutBuilder::LUT_SIZE)
      .for_each(|byte| self.inner = self.inner.wrapping_add(self.lut[byte] as u64))
  }
}

#[cfg(test)]
mod tests {
  extern crate test;

  use std::{collections::HashMap, hash::BuildHasherDefault, sync::LazyLock};

  use test::Bencher;

  use super::*;

  static DAYS: LazyLock<Lut<'static>> = LazyLock::new(|| {
    let keys = vec![
      "monday",
      "sunday",
      "tuesday",
      "saturday",
      "thursday",
      "firday",
      "wednesday",
    ];
    let mut lut = LutBuilder(keys).build();

    lut['s' as usize - 32] = 1;
    lut['t' as usize - 32] = 1;
    lut['r' as usize - 32] = 1;
    lut['h' as usize - 32] = 1;
    lut['f' as usize - 32] = 4;
    lut['w' as usize - 32] = 5;

    lut
  });

  const TEST_TEXT: &str = r#"monday is the first day of the week in many cultures, including the united states and canada. it's a busy day for most people as they begin their workweek and settle back into their routines. on tuesday many people continue their work, but others may have classes or meetings scheduled. wednesday is sometimes referred to as "hump day" because it's the middle of the workweek, and people start to look forward to the weekend. thursday are often a day for meetings and deadlines as people try to finish up their work before the end of the week. friday are a popular day for social events, happy hours, and winding down after a long workweek. saturday and sunday are usually reserved for relaxation, spending time with family and friends, and pursuing hobbies and interests. a hashtable can be a useful tool for keeping track of appointments, deadlines, and events on different days of the week."#;

  static RESULTS: LazyLock<Vec<Option<i32>>> = LazyLock::new(|| {
    let mut hashmap = HashMap::new();

    hashmap.insert("sunday", 1);
    hashmap.insert("monday", 2);
    hashmap.insert("tuesday", 3);
    hashmap.insert("wednesday", 4);
    hashmap.insert("thursday", 5);
    hashmap.insert("firday", 6);
    hashmap.insert("saturday", 7);

    TEST_TEXT
      .split(' ')
      .map(|word| hashmap.get(word).copied())
      .collect()
  });

  impl Default for VfhmHasher<'_> {
    fn default() -> Self {
      Self::new(&DAYS)
    }
  }

  #[test]
  fn basic() {
    let printable_set = (32..127).map(char::from).collect::<String>();

    let lut = LutBuilder(vec![&printable_set]).build();

    let vfhm: Vfhm<String, 1> = Vfhm::new(&lut);

    assert_eq!(vfhm.get(&printable_set).as_ref(), None);
  }

  #[test]
  fn week_days() {
    let mut vfhm: Vfhm<_, 7> = Vfhm::new(&DAYS);

    vfhm.insert("sunday", 1);
    vfhm.insert("monday", 2);
    vfhm.insert("tuesday", 3);
    vfhm.insert("wednesday", 4);
    vfhm.insert("thursday", 5);
    vfhm.insert("firday", 6);
    vfhm.insert("saturday", 7);

    println!("sunday {:?}", vfhm.get("sunday"));
    println!("monday {:?}", vfhm.get("monday"));
    println!("tuesday {:?}", vfhm.get("tuesday"));
    println!("wednesday {:?}", vfhm.get("wednesday"));
    println!("thursday {:?}", vfhm.get("thursday"));
    println!("firday {:?}", vfhm.get("firday"));
    println!("saturday {:?}", vfhm.get("saturday"));

    for (i, word) in TEST_TEXT.split(' ').enumerate() {
      if let Some(index) = vfhm.get(word) {
        println!("{i}: {word} -> {index}");
      }
    }
  }

  #[bench]
  fn bench_fnv(b: &mut Bencher) {
    let mut fnvmap = fnv::FnvHashMap::default();

    fnvmap.insert("sunday", 1);
    fnvmap.insert("monday", 2);
    fnvmap.insert("tuesday", 3);
    fnvmap.insert("wednesday", 4);
    fnvmap.insert("thursday", 5);
    fnvmap.insert("firday", 6);
    fnvmap.insert("saturday", 7);

    let text_iter = TEST_TEXT.split(' ').zip(RESULTS.iter()).collect::<Vec<_>>();

    b.iter(|| {
      text_iter.iter().for_each(|(word, result)| {
        assert_eq!(fnvmap.get(word), result.as_ref());
      });
    });
  }

  #[bench]
  fn bench_hashmap(b: &mut Bencher) {
    let mut hashmap = HashMap::new();

    hashmap.insert("sunday", 1);
    hashmap.insert("monday", 2);
    hashmap.insert("tuesday", 3);
    hashmap.insert("wednesday", 4);
    hashmap.insert("thursday", 5);
    hashmap.insert("firday", 6);
    hashmap.insert("saturday", 7);

    let text_iter = TEST_TEXT.split(' ').zip(RESULTS.iter()).collect::<Vec<_>>();

    b.iter(|| {
      text_iter.iter().for_each(|(word, result)| {
        assert_eq!(hashmap.get(word), result.as_ref(), "Failed on word {word}");
      });
    });
  }

  #[bench]
  fn bench_phf(b: &mut Bencher) {
    static KEYWORDS: phf::Map<&'static str, i32> = phf::phf_map! {
        "sunday" => 1,
        "monday" => 2,
        "tuesday" => 3,
        "wednesday" => 4,
        "thursday" => 5,
        "firday" => 6,
        "saturday" => 7,
    };

    let text_iter = TEST_TEXT.split(' ').zip(RESULTS.iter()).collect::<Vec<_>>();

    b.iter(|| {
      text_iter.iter().for_each(|(word, result)| {
        assert_eq!(KEYWORDS.get(word), result.as_ref(), "Failed on word {word}");
      });
    });
  }

  #[bench]
  fn bench_vfhm(b: &mut Bencher) {
    let mut vfhm: Vfhm<_, 7> = Vfhm::new(&DAYS);

    vfhm.insert("sunday", 1);
    vfhm.insert("monday", 2);
    vfhm.insert("tuesday", 3);
    vfhm.insert("wednesday", 4);
    vfhm.insert("thursday", 5);
    vfhm.insert("firday", 6);
    vfhm.insert("saturday", 7);

    let text_iter = TEST_TEXT.split(' ').zip(RESULTS.iter()).collect::<Vec<_>>();

    b.iter(|| {
      text_iter
        .iter()
        .enumerate()
        .for_each(|(index, (word, result))| {
          assert_eq!(
            vfhm.get(word),
            result.as_ref(),
            "Failed on word {word} at index {index}"
          );
        });
    });
  }

  #[bench]
  fn bench_vfhm_hasher(b: &mut Bencher) {
    let mut vfhm: HashMap<&str, i32, _> =
      HashMap::with_hasher(BuildHasherDefault::<VfhmHasher>::default());

    vfhm.insert("sunday", 1);
    vfhm.insert("monday", 2);
    vfhm.insert("tuesday", 3);
    vfhm.insert("wednesday", 4);
    vfhm.insert("thursday", 5);
    vfhm.insert("firday", 6);
    vfhm.insert("saturday", 7);

    let text_iter = TEST_TEXT.split(' ').zip(RESULTS.iter()).collect::<Vec<_>>();

    b.iter(|| {
      text_iter.iter().for_each(|(word, result)| {
        assert_eq!(vfhm.get(word), result.as_ref(), "Failed on word {word}");
      });
    });
  }
}