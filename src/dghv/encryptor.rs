use crate::{
    dghv::Ciphertext,
    utils::random::{Randomizer, new_rand_state},
};
use rand::seq::IteratorRandom;
use rug::{Complete, Integer};

/// DGHV [Encryptor].
/// Allows the creation of [Ciphertext] with encrypted data.
#[derive(Debug)]
pub struct Encryptor {
    /// Public key $pk$. A collection of integers.
    pk: Vec<Integer>,
    /// $\rho\'$ parameter. Secondary noise parameter. Constraint: $\rho\' = \rho + \omega(\log\lambda)$.
    big_noise_width: u16,
    /// $\tau$ parameter. Number of integers in the public key. Constraint: $\tau \geq \gamma + \omega(\log\lambda)$.
    pk_count: u32,
}

impl Encryptor {
    /// Create a new [Encryptor] with the given public key $pk$ and
    /// the parameters $\rho\'$ and $\tau$.
    pub fn new(pk: Vec<Integer>, big_noise_width: u16, pk_count: u32) -> Self {
        Encryptor {
            pk,
            big_noise_width,
            pk_count,
        }
    }

    /// Cipher a boolean value to obtain a fresh [Ciphertext].
    pub fn encrypt(&self, val: bool) -> Ciphertext {
        let subset_size = Randomizer::new().random_u32() % self.pk_count;
        let r_bound: Integer = Integer::from(1) << (self.big_noise_width as u32 + 1);
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
        let (_, sum) = sum.div_rem_round_ref(&self.pk[0]).complete(); // Change from modulo_mut
        Ciphertext::from(sum)
    }
}
