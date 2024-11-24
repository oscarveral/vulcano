use core::ops::AddAssign;

use crate::traits::{primitive::Unsigned, size::Fixed};

use super::Natural;

impl<T: Unsigned + Fixed> AddAssign<Self> for Natural<T> {
    fn add_assign(&mut self, rhs: Self) {
        let mut carry = T::zero();
    }
}
