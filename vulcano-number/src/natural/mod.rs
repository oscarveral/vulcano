mod addition;

use alloc::vec::Vec;

use crate::traits::{primitive::Unsigned, size::Dynamic, size::Fixed};

struct Natural<T: Unsigned + Fixed> {
    limbs: Vec<T>,
}

impl<T: Unsigned + Fixed> Dynamic for Natural<T> {
    fn trim_zeros(&mut self) {
        while let Some(&last) = self.limbs.last() {
            if last == T::zero() {
                self.limbs.pop();
            } else {
                break;
            }
        }
    }
}

impl<T: Unsigned + Fixed> From<T> for Natural<T> {
    fn from(value: T) -> Self {
        let mut limbs = Vec::new();
        limbs.push(value);
        Self { limbs }
    }
}
