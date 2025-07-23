use crate::{
    dghv::Ciphertext,
    dghv::random::{Randomizer, new_rand_state},
};
use rand::{RngCore, seq::IteratorRandom};
use rug::{Complete, Integer};

/// DGHV [Encryptor].
/// Allows the creation of [Ciphertext] with encrypted boolean values.
#[derive(Debug)]
pub struct Encryptor {
    /// Public key $pk$. A collection of integers.
    pk: Vec<Integer>,
    /// $\rho\'$ parameter. Secondary noise parameter. Constraint: $\rho\' = \rho + \omega(\log\lambda)$.
    rho_prime: u16,
    /// $\tau$ parameter. Number of integers in the public key. Constraint: $\tau \geq \gamma + \omega(\log\lambda)$.
    tau: u32,
}

impl Encryptor {
    /// Create a new [Encryptor] with the given public key $pk$ and
    /// the parameters $\rho\'$ and $\tau$.
    pub fn new(pk: Vec<Integer>, rho_prime: u16, tau: u32) -> Self {
        Encryptor { pk, rho_prime, tau }
    }

    /// Cipher a boolean value to get a fresh [Ciphertext].
    pub fn encrypt(&self, val: bool) -> Ciphertext {
        let subset_size = Randomizer::new().next_u32() % self.tau;
        let r_bound: Integer = Integer::from(1) << (self.rho_prime as u32 + 1);
        let mut r: Integer = Integer::from(0);
        while r == 0 {
            r = r_bound.random_below_ref(&mut new_rand_state()).complete()
        }
        r -= r_bound >> 1;
        let subset = self.pk[1..]
            .iter()
            .choose_multiple(&mut Randomizer::new(), subset_size as usize);
        let mut sum: Integer = subset.into_iter().sum();
        sum *= 2;
        r *= 2;
        sum += r;
        sum += val as u8;
        let (_, sum) = sum.div_rem_round_ref(&self.pk[0]).complete();
        Ciphertext::from(sum)
    }

    /// Get the memory footprint in bytes of the [Encryptor].
    pub fn get_size(&self) -> usize {
        let mut size = size_of_val(self);
        for int in &self.pk {
            size += int.capacity() / (u8::BITS as usize);
            size += size_of::<Integer>();
        }
        size
    }
}
