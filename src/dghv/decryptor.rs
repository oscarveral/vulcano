use rug::{Complete, Integer};

use crate::dghv::Ciphertext;

/// DGHV [Decryptor].
/// Allows the decryption of boolean values stored in ciphertext data.
#[derive(Debug)]
pub struct Decryptor {
    /// Secret key $sk$. Allows decryption of [Ciphertext]
    sk: Integer,
}

impl Decryptor {
    /// Create a new [Decryptor] with the given secret key.
    pub fn new(sk: Integer) -> Self {
        Decryptor { sk }
    }

    /// Retrieve the boolean value stored in a given ciphertext.
    pub fn decrypt(&self, val: Ciphertext) -> bool {
        let centered_remainder = <Ciphertext as Into<Integer>>::into(val)
            .div_rem_round_ref(&self.sk)
            .complete()
            .1;
        let res = centered_remainder.modulo(&Integer::from(2));
        !res.is_zero()
    }

    /// Get the memory footprint in bytes of the [Decryptor].
    pub fn get_size(&self) -> usize {
        let size = size_of_val(self);
        size + (self.sk.capacity() / (u8::BITS as usize))
    }
}
