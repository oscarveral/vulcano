use crate::dghv::ciphertext::Ciphertext;
use crate::dghv::distribution::{NoiseSampler, SubsetSampler};
use crate::dghv::math::remainder;
use rug::Integer;

/// [Encryptor] used to create fresh [ciphertexts](Ciphertext)
pub struct Encryptor {
    /// Public key elements.
    pk: Vec<Integer>,
    /// Sampler for sparse subsets.
    subset_sampler: SubsetSampler,
    /// Sampler for noise.
    noise_sampler: NoiseSampler,
}

impl Encryptor {
    /// Create a new [Encryptor] using the given public key and $\tau$ parameter.
    pub fn new(pk: Vec<Integer>, tau: u32, rho_prime: u16) -> Self {
        let pk_size: usize = tau
            .try_into()
            .unwrap_or_else(|_| panic!("Param τ overflow!"));
        let subset_sampler = SubsetSampler::new(pk_size);
        let noise_sampler = NoiseSampler::new(rho_prime);
        Self {
            pk,
            subset_sampler,
            noise_sampler,
        }
    }

    /// Encrypt a boolean value into a [Ciphertext].
    pub fn encrypt(&mut self, data: bool) -> Ciphertext {
        let subset = self.subset_sampler.sample_index_subset();
        let noise = self.noise_sampler.sample_noise();

        let mut val = Integer::from(data);

        val += noise * 2;
        let mut sum = Integer::from(0);
        for i in subset {
            sum += &self.pk[i];
        }
        val += sum * 2;

        remainder(&val, &self.pk[0]).into()
    }
}
