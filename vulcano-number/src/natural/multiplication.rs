use core::ops::MulAssign;

use crate::{
    natural::Natural,
    traits::{primitive::Unsigned, size::Fixed},
};

impl<T: Unsigned + Fixed> MulAssign<T> for Natural<T> {
    fn mul_assign(&mut self, rhs: T) {}
}
