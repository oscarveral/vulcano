use core::ops::AddAssign;

use crate::traits::{primitive::Unsigned, size::Fixed};

use crate::natural::Natural;

impl<T: Unsigned + Fixed> AddAssign<T> for Natural<T> {
    fn add_assign(&mut self, rhs: T) {
        for limb in &mut self.limbs {}
    }
}
