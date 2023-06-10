pub trait VfhmConstParams {
  const SEED: usize;
  const MASK: usize;
  const MASK_OFFSET: usize;

  fn into_params() -> VfhmParams {
    VfhmParams(Self::SEED, Self::MASK, Self::MASK_OFFSET)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct VfhmParams(pub usize, pub usize, pub usize);

impl VfhmParams {
  pub fn mask_size(&self) -> usize {
    let VfhmParams(_, mask, mask_offset) = *self;
    (mask >> mask_offset) + 1
  }
}

impl<T> From<T> for VfhmParams
where
  T: VfhmConstParams,
{
  fn from(_: T) -> Self {
    VfhmParams(T::SEED, T::MASK, T::MASK_OFFSET)
  }
}

pub struct VfhmDefaultParams;

impl VfhmConstParams for VfhmDefaultParams {
  const SEED: usize = 0;
  const MASK: usize = 1;
  const MASK_OFFSET: usize = 0;
}
