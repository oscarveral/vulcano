use crate::dghv::ciphertext::Ciphertext;
use crate::dghv::math::remainder;
use rug::Integer;

/// [Evaluator] used to compute functions over [Ciphertext].
pub struct Evaluator {
    /// Downsize key.
    dsk: Integer,
}

impl Evaluator {
    /// Create a new [Evaluator] using the given downsize key.
    pub fn new(dsk: Integer) -> Self {
        Self { dsk }
    }

    /// Compute the sum of the given [Ciphertext].
    pub fn add(&self, a: Ciphertext, b: Ciphertext) -> Ciphertext {
        let raw_a: Integer = a.into();
        let raw_b: Integer = b.into();
        (raw_a + raw_b).into()
    }

    /// Compute the multiplication of the given [Ciphertext].
    pub fn mul(&self, a: Ciphertext, b: Ciphertext) -> Ciphertext {
        let raw_a: Integer = a.into();
        let raw_b: Integer = b.into();
        (raw_a * raw_b).into()
    }

    /// Reduce the size of the given [Ciphertext].
    pub fn downsize(&self, c: Ciphertext) -> Ciphertext {
        let raw: Integer = c.into();
        remainder(&raw, &self.dsk).into()
    }
}
