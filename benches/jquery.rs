#![feature(once_cell)]

use std::{collections::HashMap, sync::LazyLock};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fnv::FnvHashMap;

macro_rules! add_keywords {
  ($ident:ident) => {
    $ident.insert("await", 1);
    $ident.insert("break", 2);
    $ident.insert("case", 3);
    $ident.insert("catch", 4);
    $ident.insert("class", 5);
    $ident.insert("const", 6);
    $ident.insert("continue", 7);
    $ident.insert("debugger", 8);
    $ident.insert("default", 9);
    $ident.insert("delete", 10);
    $ident.insert("do", 11);
    $ident.insert("else", 12);
    $ident.insert("enum", 13);
    $ident.insert("export", 14);
    $ident.insert("extends", 15);
    $ident.insert("false", 16);
    $ident.insert("finally", 17);
    $ident.insert("for", 18);
    $ident.insert("function", 19);
    $ident.insert("if", 20);
    $ident.insert("implements", 21);
    $ident.insert("import", 22);
    $ident.insert("in", 23);
    $ident.insert("instanceof", 24);
    $ident.insert("interface", 25);
    $ident.insert("let", 26);
    $ident.insert("new", 27);
    $ident.insert("null", 28);
    $ident.insert("package", 29);
    $ident.insert("private", 30);
    $ident.insert("protected", 31);
    $ident.insert("public", 32);
    $ident.insert("return", 33);
    $ident.insert("super", 34);
    $ident.insert("switch", 35);
    $ident.insert("static", 36);
    $ident.insert("this", 37);
    $ident.insert("throw", 38);
    $ident.insert("try", 39);
    $ident.insert("true", 40);
    $ident.insert("typeof", 41);
    $ident.insert("var", 42);
    $ident.insert("void", 43);
    $ident.insert("while", 44);
    $ident.insert("with", 45);
    $ident.insert("yield", 46);
  };
}

const TEST_TEXT: &str = include_str!("./jquery-3.6.4.js");

static TEXT_VALUES: LazyLock<Vec<(&str, Option<i32>)>> = LazyLock::new(|| {
  let mut hashmap = HashMap::new();

  add_keywords!(hashmap);

  TEST_TEXT
    .split(' ')
    .filter(|word| !word.is_empty())
    .map(|word| (word, hashmap.get(&word).copied()))
    .collect()
});

fn bench_fnv(c: &mut Criterion) {
  let _ = *TEXT_VALUES;
  let mut hashmap = FnvHashMap::default();

  add_keywords!(hashmap);

  c.bench_with_input(BenchmarkId::new("fnv", "jquery"), &hashmap, |b, hashmap| {
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

  add_keywords!(hashmap);

  c.bench_with_input(
    BenchmarkId::new("hashmap", "jquery"),
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

fn bench_vfhd2(c: &mut Criterion) {
  let mut hashmap = HashMap::new();

  add_keywords!(hashmap);

  let keys: Vec<_> = hashmap.keys().collect();

  let mut hashmap = vfhm::v2::Vfhm::new(vfhm::v2::find_seed(&keys), (2, 10));

  add_keywords!(hashmap);

  c.bench_with_input(
    BenchmarkId::new("vfhm2", "jquery"),
    &hashmap,
    |b, hashmap| {
      b.iter(|| {
        let hashmap = black_box(hashmap);
        TEXT_VALUES.iter().for_each(|(word, result)| {
          assert_eq!(hashmap.get(*word), result.as_ref(), "Failed on word {word}");
        });
      });
    },
  );
}

criterion_group!(benches, bench_hashmap, bench_fnv, bench_vfhd2);
criterion_main!(benches);