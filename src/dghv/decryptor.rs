use crate::dghv::ciphertext::Ciphertext;
use crate::dghv::math::remainder;
use rug::Integer;

/// A [Decryptor] used to decrypt [Ciphertext].
pub struct Decryptor {
    /// Secret generator value used.
    generator: Integer,
}

impl Decryptor {
    /// Create a new [Decryptor] using the given generator.
    pub fn new(generator: Integer) -> Self {
        Self { generator }
    }

    /// Decrypt the given [Ciphertext] into a boolean value.
    pub fn decrypt(&self, ciphertext: Ciphertext) -> bool {
        let raw: Integer = ciphertext.into();
        let rem = remainder(&raw, &self.generator);
        remainder(&rem, &Integer::from(2)).is_odd()
    }
}
