use std::ops::{Add, AddAssign, Mul, MulAssign};

use rug::Integer;

/// DGHV [Ciphertext].
/// Store private data.
#[derive(Clone, Debug)]
pub struct Ciphertext(
    /// Internally a ciphertext is only an integer.
    Integer,
);

impl Add for Ciphertext {
    type Output = Ciphertext;

    fn add(self, rhs: Self) -> Self::Output {
        Ciphertext(self.0.add(rhs.0))
    }
}

impl AddAssign for Ciphertext {
    fn add_assign(&mut self, rhs: Self) {
        self.0.add_assign(rhs.0);
    }
}

impl Mul for Ciphertext {
    type Output = Ciphertext;

    fn mul(self, rhs: Self) -> Self::Output {
        Ciphertext(self.0.mul(rhs.0))
    }
}

impl MulAssign for Ciphertext {
    fn mul_assign(&mut self, rhs: Self) {
        self.0.mul_assign(rhs.0);
    }
}

impl From<Integer> for Ciphertext {
    fn from(value: Integer) -> Self {
        Ciphertext(value)
    }
}

impl From<Ciphertext> for Integer {
    fn from(value: Ciphertext) -> Self {
        value.0
    }
}

impl Ciphertext {
    /// Obtain the memory footprint of the [Ciphertext].
    pub fn get_size(&self) -> usize {
        let size = std::mem::size_of_val(self);
        size + (self.0.capacity() / (u8::BITS as usize))
    }
}
