use crate::dghv::decryptor::Decryptor;
use crate::dghv::distribution::{GeneratorSampler, PublicKeySampler};
use crate::dghv::encryptor::Encryptor;
use crate::dghv::evaluator::Evaluator;
use crate::dghv::parameters::Parameters;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
use rayon::slice::ParallelSliceMut;
use rug::Integer;

/// Crypto context for the DGHV Scheme. Contain all necessary sampler and pre-computations
/// needed for the generation of [Encryptor], [Decryptor] and [Evaluator] components.
pub struct Context {
    /// Params used by this [Context].
    params: Parameters,
    /// Sampler used for public key elements and downsize key.
    pk_sampler: PublicKeySampler,
}

impl Context {
    /// Create a new [Context] using the given [Parameters].
    pub fn new(params: Parameters) -> Self {
        let mut generator_sampler = GeneratorSampler::new(params.eta);
        let pk_sampler =
            PublicKeySampler::new(params.gamma, params.rho, generator_sampler.sample());

        Self { params, pk_sampler }
    }

    /// Generate new precomputations and samplers for the current [Context]
    pub fn refresh_context(&mut self) {
        let mut generator_sampler = GeneratorSampler::new(self.params.eta);
        self.pk_sampler = PublicKeySampler::new(
            self.params.gamma,
            self.params.rho,
            generator_sampler.sample(),
        );
    }

    /// Change the [Parameters] of the [Context] and refresh all precomputations and samplers.
    pub fn change_params(&mut self, new_params: Parameters) {
        self.params = new_params;
        self.refresh_context();
    }

    /// Get the current secret generator key from the [GeneratorSampler]
    pub fn generator_key_sample(&self) -> Integer {
        self.pk_sampler.get_generator()
    }

    /// Sample a downsize key using the current [Context] [PublicKeySampler] noiseless distribution.
    /// As it is the first element of the public key, it must be odd.
    pub fn downsize_key_sample(&mut self) -> Integer {
        let mut rsk = self.pk_sampler.sample_without_noise();
        while rsk.is_even() {
            rsk = self.pk_sampler.sample_without_noise();
        }
        rsk
    }

    /// Sample a public key using the current [Context] samplers.
    /// Public key is a collection of $\tau$ integers sampled from the noisy distribution specified at [PublicKeySampler]
    /// but with the first element being the downsize key, that must the largest element of the collection,
    /// must be odd, and an exact multiple of the secret generator integer $p$, i.e. sampled from the noiseless
    /// distribution of [PublicKeySampler]. Returns the public key and downsize key.
    pub fn public_key_sample(&mut self) -> (Vec<Integer>, Integer) {
        let mut pk: Vec<Integer> = vec![
            Integer::from(0);
            self.params
                .tau
                .try_into()
                .unwrap_or_else(|_| panic!("Param τ overflow!"))
        ];
        loop {
            pk[1..].par_iter_mut().for_each(|x| {
                *x = self.pk_sampler.sample_with_noise_par();
            });

            pk[1..].par_sort_unstable();
            pk[1..].reverse();

            pk[0] = self.downsize_key_sample();

            if pk[0] >= pk[1] {
                break;
            }
        }

        let dsk = pk[0].clone();
        (pk, dsk)
    }

    pub fn key_gen(&mut self) -> (Encryptor, Decryptor, Evaluator) {
        let g = self.generator_key_sample();
        let (pk, dsk) = self.public_key_sample();
        (
            Encryptor::new(pk, self.params.tau, self.params.rho_prime),
            Decryptor::new(g),
            Evaluator::new(dsk),
        )
    }
}
