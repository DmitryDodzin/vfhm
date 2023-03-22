#![feature(once_cell)]

use std::{collections::HashMap, sync::LazyLock};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fnv::FnvHashMap;
use vfhm::r#match::MatchMap;

const TEST_TEXT: &str = r#"monday is the first day of the week in many cultures, including the united states and canada. it's a busy day for most people as they begin their workweek and settle back into their routines. on tuesday many people continue their work, but others may have classes or meetings scheduled. wednesday is sometimes referred to as "hump day" because it's the middle of the workweek, and people start to look forward to the weekend. thursday are often a day for meetings and deadlines as people try to finish up their work before the end of the week. friday are a popular day for social events, happy hours, and winding down after a long workweek. saturday and sunday are usually reserved for relaxation, spending time with family and friends, and pursuing hobbies and interests. a hashtable can be a useful tool for keeping track of appointments, deadlines, and events on different days of the week."#;

static TEXT_VALUES: LazyLock<Vec<(&str, Option<i32>)>> = LazyLock::new(|| {
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
    .map(|word| (word, hashmap.get(word).copied()))
    .collect()
});

static DAYS: LazyLock<vfhm::Lut<'static>> = LazyLock::new(|| {
  let keys = vec![
    "monday",
    "sunday",
    "tuesday",
    "saturday",
    "thursday",
    "firday",
    "wednesday",
  ];
  let mut lut = vfhm::LutBuilder(keys).build();

  lut['s' as usize - 32] = 1;
  lut['t' as usize - 32] = 1;
  lut['r' as usize - 32] = 1;
  lut['h' as usize - 32] = 1;
  lut['f' as usize - 32] = 4;
  lut['w' as usize - 32] = 5;

  lut
});

#[derive(Debug, Default)]
struct DayHashMap<T>([Option<T>; 7]);

impl<T> DayHashMap<T> {
  #[inline]
  fn get(&self, key: &str) -> Option<&T> {
    let DayHashMap(values) = self;

    match key {
      "sunday" => values[0].as_ref(),
      "monday" => values[1].as_ref(),
      "tuesday" => values[2].as_ref(),
      "wednesday" => values[3].as_ref(),
      "thursday" => values[4].as_ref(),
      "firday" => values[5].as_ref(),
      "saturday" => values[6].as_ref(),
      _ => None,
    }
  }

  #[inline]
  fn insert(&mut self, key: &str, value: T) -> Option<T> {
    let DayHashMap(ref mut values) = self;

    let mut output = Some(value);

    match key {
      "sunday" => std::mem::swap(&mut output, &mut values[0]),
      "monday" => std::mem::swap(&mut output, &mut values[1]),
      "tuesday" => std::mem::swap(&mut output, &mut values[2]),
      "wednesday" => std::mem::swap(&mut output, &mut values[3]),
      "thursday" => std::mem::swap(&mut output, &mut values[4]),
      "firday" => std::mem::swap(&mut output, &mut values[5]),
      "saturday" => std::mem::swap(&mut output, &mut values[6]),
      _ => {
        output = None;
      }
    }

    output
  }
}
vfhm::match_map! { LinearMatcher<&'static str>[] }
vfhm::match_map! { DaysMatcher<&'static str>["sunday", "monday", "tuesday", "wednesday", "thursday", "firday", "saturday"] }

type LinearMatchMap<V> = MatchMap<LinearMatcher, V>;
type DaysMatchMap<V> = MatchMap<DaysMatcher, V>;

fn bench_fnv(c: &mut Criterion) {
  let _ = *TEXT_VALUES;
  let mut hashmap = FnvHashMap::default();

  hashmap.insert("sunday", 1);
  hashmap.insert("monday", 2);
  hashmap.insert("tuesday", 3);
  hashmap.insert("wednesday", 4);
  hashmap.insert("thursday", 5);
  hashmap.insert("firday", 6);
  hashmap.insert("saturday", 7);

  c.bench_with_input(BenchmarkId::new("fnv", "days"), &hashmap, |b, hashmap| {
    b.iter(|| {
      let hashmap = black_box(hashmap);

      TEXT_VALUES.iter().for_each(|(word, result)| {
        assert_eq!(hashmap.get(word), result.as_ref(), "Failed on word {word}");
      });
    });
  });
}

fn bench_hashmap(c: &mut Criterion) {
  let _ = *TEXT_VALUES;
  let mut hashmap = HashMap::new();

  hashmap.insert("sunday", 1);
  hashmap.insert("monday", 2);
  hashmap.insert("tuesday", 3);
  hashmap.insert("wednesday", 4);
  hashmap.insert("thursday", 5);
  hashmap.insert("firday", 6);
  hashmap.insert("saturday", 7);

  c.bench_with_input(
    BenchmarkId::new("hashmap", "days"),
    &hashmap,
    |b, hashmap| {
      b.iter(|| {
        let hashmap = black_box(hashmap);

        TEXT_VALUES.iter().for_each(|(word, result)| {
          assert_eq!(hashmap.get(word), result.as_ref(), "Failed on word {word}");
        });
      });
    },
  );
}

fn bench_match(c: &mut Criterion) {
  let _ = *TEXT_VALUES;
  let mut hashmap = DayHashMap::default();

  hashmap.insert("sunday", 1);
  hashmap.insert("monday", 2);
  hashmap.insert("tuesday", 3);
  hashmap.insert("wednesday", 4);
  hashmap.insert("thursday", 5);
  hashmap.insert("firday", 6);
  hashmap.insert("saturday", 7);

  c.bench_with_input(BenchmarkId::new("match", "days"), &hashmap, |b, hashmap| {
    b.iter(|| {
      let hashmap = black_box(hashmap);
      TEXT_VALUES.iter().for_each(|(word, result)| {
        assert_eq!(hashmap.get(word), result.as_ref(), "Failed on word {word}");
      });
    });
  });
}

fn bench_match2(c: &mut Criterion) {
  let _ = *TEXT_VALUES;
  let mut hashmap = DaysMatchMap::default();

  hashmap.insert("sunday", 1);
  hashmap.insert("monday", 2);
  hashmap.insert("tuesday", 3);
  hashmap.insert("wednesday", 4);
  hashmap.insert("thursday", 5);
  hashmap.insert("firday", 6);
  hashmap.insert("saturday", 7);

  c.bench_with_input(
    BenchmarkId::new("match2", "days"),
    &hashmap,
    |b, hashmap| {
      b.iter(|| {
        let hashmap = black_box(hashmap);
        TEXT_VALUES.iter().for_each(|(word, result)| {
          assert_eq!(hashmap.get(word), result.as_ref(), "Failed on word {word}");
        });
      });
    },
  );
}

fn bench_linear(c: &mut Criterion) {
  let _ = *TEXT_VALUES;
  let mut hashmap = LinearMatchMap::default();

  hashmap.insert("sunday", 1);
  hashmap.insert("monday", 2);
  hashmap.insert("tuesday", 3);
  hashmap.insert("wednesday", 4);
  hashmap.insert("thursday", 5);
  hashmap.insert("firday", 6);
  hashmap.insert("saturday", 7);

  c.bench_with_input(
    BenchmarkId::new("match_linear", "days"),
    &hashmap,
    |b, hashmap| {
      b.iter(|| {
        let hashmap = black_box(hashmap);
        TEXT_VALUES.iter().for_each(|(word, result)| {
          assert_eq!(hashmap.get(word), result.as_ref(), "Failed on word {word}");
        });
      });
    },
  );
}

fn bench_vfhd(c: &mut Criterion) {
  let _ = *TEXT_VALUES;
  let mut hashmap = vfhm::Vfhm::<'_, i32, 7>::new(&DAYS);

  hashmap.insert("sunday", 1);
  hashmap.insert("monday", 2);
  hashmap.insert("tuesday", 3);
  hashmap.insert("wednesday", 4);
  hashmap.insert("thursday", 5);
  hashmap.insert("firday", 6);
  hashmap.insert("saturday", 7);

  c.bench_with_input(BenchmarkId::new("vfhm", "days"), &hashmap, |b, hashmap| {
    b.iter(|| {
      let hashmap = black_box(hashmap);
      TEXT_VALUES.iter().for_each(|(word, result)| {
        assert_eq!(hashmap.get(word), result.as_ref(), "Failed on word {word}");
      });
    });
  });
}

fn bench_vfhd2(c: &mut Criterion) {
  let _ = *TEXT_VALUES;
  let keys = vec![
    "sunday",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "firday",
    "saturday",
  ];

  let mut hashmap = vfhm::v2::Vfhm::new(vfhm::v2::find_seed(&keys), (6, 9));

  hashmap.insert("sunday", 1);
  hashmap.insert("monday", 2);
  hashmap.insert("tuesday", 3);
  hashmap.insert("wednesday", 4);
  hashmap.insert("thursday", 5);
  hashmap.insert("firday", 6);
  hashmap.insert("saturday", 7);

  c.bench_with_input(BenchmarkId::new("vfhm2", "days"), &hashmap, |b, hashmap| {
    b.iter(|| {
      let hashmap = black_box(hashmap);
      TEXT_VALUES.iter().for_each(|(word, result)| {
        assert_eq!(hashmap.get(*word), result.as_ref(), "Failed on word {word}");
      });
    });
  });
}

criterion_group!(
  benches,
  bench_hashmap,
  bench_fnv,
  bench_match,
  bench_match2,
  bench_linear,
  bench_vfhd,
  bench_vfhd2,
);
criterion_main!(benches);
