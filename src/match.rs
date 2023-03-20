use std::borrow::Borrow;

pub trait MatchMapMatcher {
  type Key: PartialEq;

  fn keys() -> Vec<Self::Key>;

  fn match_key(key: &Self::Key) -> Option<usize>;
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
