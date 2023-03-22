use std::borrow::Borrow;

pub trait MatchMapMatcher {
  type Key: PartialEq;

  fn keys() -> Vec<Self::Key>;

  fn match_key(key: &Self::Key) -> Option<usize>;
}

#[macro_export]
macro_rules! match_map {
  { $ident:ident <$ty:ty> [$($lit:literal),*] } => {
    struct $ident;

    impl vfhm::r#match::MatchMapMatcher for $ident {
      type Key = $ty;

      fn keys() -> Vec<Self::Key> {
        vec![$($lit),*]
      }

      #[inline]
      fn match_key(key: &Self::Key) -> Option<usize> {
        vfhm::match_map_key!(key; 0; $($lit),*)
      }
    }
  };
}

#[macro_export]
macro_rules! match_map_key {
  ($expr:expr; $curr:expr;) => {
    None
  };
  ($expr:expr; $curr:expr; $lit:literal) => {
    if let &$lit = $expr {
      Some($curr)
    } else {
      None
    }
  };
  ($expr:expr; $curr:expr; $lit:literal, $($rest:literal),*) => {
    if let &$lit = $expr {
        Some($curr)
    } else {
        vfhm::match_map_key! ($expr; $curr+1; $($rest),*)
    }
  };
}

pub struct MatchMap<M: MatchMapMatcher, V> {
  values: Vec<Option<V>>,

  liner: Vec<(M::Key, V)>,
}

impl<M, V> MatchMap<M, V>
where
  M: MatchMapMatcher,
{
  pub fn new() -> Self {
    let keys = M::keys();

    let values = vec![(); keys.len()].iter().map(|_| None).collect();

    MatchMap {
      values,
      liner: vec![],
    }
  }

  pub fn get<Q>(&self, key: &Q) -> Option<&V>
  where
    Q: Borrow<M::Key>,
    M::Key: PartialEq<Q>,
  {
    match M::match_key(key.borrow()) {
      Some(index) => self.values[index].as_ref(),
      None => self.liner.iter().find(|(k, _)| k == key).map(|(_, v)| v),
    }
  }

  pub fn insert(&mut self, key: M::Key, value: V) -> Option<V> {
    match M::match_key(&key) {
      Some(index) => {
        let mut output = Some(value);

        std::mem::swap(&mut self.values[index], &mut output);

        output
      }
      None => {
        self.liner.push((key, value));

        None
      }
    }
  }
}

impl<M, V> Default for MatchMap<M, V>
where
  M: MatchMapMatcher,
{
  fn default() -> Self {
    MatchMap::new()
  }
}
