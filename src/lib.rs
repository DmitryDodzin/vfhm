#![feature(test)]

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Vfhm<'lut, T, const SIZE: usize> {
  keys: &'lut [&'lut str],
  lut: &'lut [usize],
  inner: [Option<T>; SIZE],
}

impl<T, const SIZE: usize> Vfhm<'_, T, SIZE> {
  pub fn get<K: VfhmKey>(&self, key: K) -> &Option<T> {
    let index = key.key_index(self.lut);

    if index < SIZE && key.is_same_key(self.keys[index]) {
      &self.inner[index]
    } else {
      &None
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

impl LutBuilder<'_> {
  pub const LUT_SIZE: usize = 96;

  pub fn build(self) -> [usize; LutBuilder::LUT_SIZE] {
    [0; LutBuilder::LUT_SIZE]
  }
}

#[cfg(test)]
mod tests {
  extern crate test;

  use std::collections::HashMap;

  use test::Bencher;

  use super::*;

  const TEST_TEXT: &str = r#"monday is the first day of the week in many cultures, including the united states and canada. it's a busy day for most people as they begin their workweek and settle back into their routines. on tuesdays, many people continue their work, but others may have classes or meetings scheduled. wednesday is sometimes referred to as "hump day" because it's the middle of the workweek, and people start to look forward to the weekend. thursdays are often a day for meetings and deadlines as people try to finish up their work before the end of the week. fridays are a popular day for social events, happy hours, and winding down after a long workweek. saturdays and sundays are usually reserved for relaxation, spending time with family and friends, and pursuing hobbies and interests. a hashtable can be a useful tool for keeping track of appointments, deadlines, and events on different days of the week."#;

  #[test]
  fn basic() {
    let printable_set = (32..127).map(char::from).collect::<String>();

    let lut = LutBuilder(vec![&printable_set]).build();

    let vfhm: Vfhm<String, 1> = Vfhm {
      keys: &[&printable_set],
      inner: [None],
      lut: &lut,
    };

    assert_eq!(vfhm.get(&printable_set).as_ref(), None);
  }

  #[test]
  fn week_days() {
    let keys = [
      "monday",
      "sunday",
      "tuesday",
      "saturday",
      "thursday",
      "firday",
      "wednesday",
    ];
    let mut lut = LutBuilder(keys.to_vec()).build();

    lut['s' as usize - 32] = 1;
    lut['t' as usize - 32] = 1;
    lut['r' as usize - 32] = 1;
    lut['h' as usize - 32] = 1;
    lut['f' as usize - 32] = 4;
    lut['w' as usize - 32] = 5;

    let mut vfhm: Vfhm<_, 7> = Vfhm {
      keys: &keys,
      inner: [None; 7],
      lut: &lut,
    };

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
  fn bench_vfhd(b: &mut Bencher) {
    let keys = [
      "sunday",
      "monday",
      "tuesday",
      "wednesday",
      "thursday",
      "firday",
      "saturday",
    ];
    let mut lut = LutBuilder(keys.to_vec()).build();

    lut['s' as usize - 32] = 1;
    lut['t' as usize - 32] = 1;
    lut['r' as usize - 32] = 1;
    lut['h' as usize - 32] = 1;
    lut['f' as usize - 32] = 4;
    lut['w' as usize - 32] = 5;

    let mut vfhm: Vfhm<_, 7> = Vfhm {
      keys: &keys,
      inner: [None; 7],
      lut: &lut,
    };

    vfhm.insert("sunday", 1);
    vfhm.insert("monday", 2);
    vfhm.insert("tuesday", 3);
    vfhm.insert("wednesday", 4);
    vfhm.insert("thursday", 5);
    vfhm.insert("firday", 6);
    vfhm.insert("saturday", 7);

    let mut text_iter = TEST_TEXT.split(' ').collect::<Vec<_>>().into_iter().cycle();

    b.iter(|| {
      text_iter.next().map(|word| vfhm.get(word));
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

    let mut text_iter = TEST_TEXT.split(' ').collect::<Vec<_>>().into_iter().cycle();

    b.iter(|| {
      text_iter.next().map(|word| hashmap.get(word));
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

    let mut text_iter = TEST_TEXT.split(' ').collect::<Vec<_>>().into_iter().cycle();

    b.iter(|| {
      text_iter.next().map(|word| KEYWORDS.get(word));
    });
  }
}
