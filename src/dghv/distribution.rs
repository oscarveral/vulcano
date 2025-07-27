use rand::seq::IteratorRandom;
use rug::{Complete, Integer, ops::DivRounding, rand::RandState};

use crate::dghv::random::{Randomizer, new_rand_state};

/// Sampler used to obtain random numbers to be used as public key and downsize elements.
/// It implements the distribution $\mathcal{D}_{\gamma,\rho}(p)$.
pub struct PublicKeySampler {
    /// Secret integer generator $p$ to be used on sample calculation.
    generator: Integer,
    /// Bound on the generator multiple range. $2^\gamma/p$. Adjusted based on $p$ generator value.
    multiple_bound: Integer,
    /// Bound on the noise generation range. $2^{\rho + 1}$. Noise will be adjusted to range $(-2^\rho, 2^\rho)$.
    noise_bound: Integer,
    /// Randomness source.
    rng: RandState<'static>,
}

impl PublicKeySampler {
    /// Create a new [PublicKeySampler] based on the $\gamma$ and $\rho$ supplied parameters.
    /// Sampled elements will be directly correlated to this params and the given secret generator integer $p$.
    pub fn new(gamma: u32, rho: u16, generator: Integer) -> Self {
        let rng = new_rand_state();
        let noise_bound = Integer::from(1)
            << (rho
                .checked_add(1)
                .unwrap_or_else(|| panic!("Parameter ρ overflow!")) as u32);
        let multiple_bound = (Integer::from(1) << gamma).div_ceil(&generator);

        Self {
            generator,
            multiple_bound,
            noise_bound,
            rng,
        }
    }

    /// Retrieve the secret generator used on this distribution.
    pub fn get_generator(&self) -> Integer {
        self.generator.clone()
    }

    /// Using the store secret generator integer $p$, sample a random noisy element in the form of $x=pq+r$
    /// with $q\leftarrow\mathbb{Z}\cap[0,2^\gamma/p)$ and $r\leftarrow\mathbb{Z}\cap(-2^\rho,2^\rho)$.
    pub fn sample_with_noise(&mut self) -> Integer {
        let q = self
            .multiple_bound
            .random_below_ref(&mut self.rng)
            .complete();
        let mut r: Integer = Integer::from(0);
        while r == 0 {
            r = self.noise_bound.random_below_ref(&mut self.rng).complete()
        }
        r -= (&self.noise_bound >> 1u32).complete();
        (q * &self.generator) + r
    }

    /// Using the store secret generator integer $p$, sample a random noisy element in the form of $x=pq+r$
    /// with $q\leftarrow\mathbb{Z}\cap[0,2^\gamma/p)$ and $r\leftarrow\mathbb{Z}\cap(-2^\rho,2^\rho)$.
    /// Usable in parallel contexts.
    pub fn sample_with_noise_par(&self) -> Integer {
        let rng = &mut new_rand_state();
        let q = self.multiple_bound.random_below_ref(rng).complete();
        let mut r: Integer = Integer::from(0);
        while r == 0 {
            r = self.noise_bound.random_below_ref(rng).complete()
        }
        r -= (&self.noise_bound >> 1u32).complete();
        (q * &self.generator) + r
    }

    /// Using the store secret generator integer $p$, sample a random noiseless element in the form of $x=pq$
    /// with $q\leftarrow\mathbb{Z}\cap[0,2^\gamma/p)$. $x$ will be an exact multiple of the secret generator $p$.
    pub fn sample_without_noise(&mut self) -> Integer {
        let q = self
            .multiple_bound
            .random_below_ref(&mut self.rng)
            .complete();
        q * &self.generator
    }
}

/// Sampler used on the encryption of values. Samples random numbers on the
/// $(-2^{\rho\'}, 2^{\rho\'}) range.
pub struct NoiseSampler {
    /// Bound on the generated samples. Base on $\rho\'$ parameter.
    bound: Integer,
    /// Randomness source.
    rng: RandState<'static>,
}

impl NoiseSampler {
    /// Create a new [NoiseSampler] using the given $\rho\'$ parameter.
    pub fn new(rho_prime: u16) -> Self {
        let bound = Integer::from(1)
            << (rho_prime
                .checked_add(1)
                .unwrap_or_else(|| panic!("Parameter ρ′ overflow!")) as u32);
        let rng = new_rand_state();
        Self { bound, rng }
    }

    /// Sample a new value from the $(-2^{\rho\'}, 2^{\rho\'}) range.
    pub fn sample_noise(&mut self) -> Integer {
        let mut r: Integer = Integer::from(0);
        while r == 0 {
            r = self.bound.random_below_ref(&mut self.rng).complete()
        }
        r -= (&self.bound >> 1u32).complete();
        r
    }
}

/// Sampler used to obtain secret integer generator samples.
/// It samples from the distribution $p\leftarrow(2\mathbb{Z}+1)\cap[2^{\eta-1}, 2^\eta)$.
pub struct GeneratorSampler {
    /// Bound on the samples range. Odd integers on the range $[2^{\eta-1}, 2^\eta)$.
    bound: Integer,
    /// Source of randomness.
    rng: RandState<'static>,
}

impl GeneratorSampler {
    /// Create a new [GeneratorSampler] integer sampler using the given $\eta$ parameter.
    pub fn new(eta: u32) -> Self {
        let rng = new_rand_state();
        let bound = Integer::from(1)
            << eta
                .checked_sub(1)
                .unwrap_or_else(|| panic!("Parameter η underflow!"));

        Self { bound, rng }
    }

    /// Generate a generator key $p$.  This function takes a sample
    /// $p\leftarrow(2\mathbb{Z}+1)\cap[2^{\eta-1}, 2^\eta)$.
    pub fn sample(&mut self) -> Integer {
        let sample: Integer =
            (self.bound.random_below_ref(&mut self.rng).complete() + &self.bound) | 0x1;
        sample
    }
}

/// Sampler used to get sparse subset samples.
pub struct SubsetSampler {
    /// Source of randomness.
    rng: Randomizer,
    /// Size of the set.
    set_size: usize,
}

impl SubsetSampler {
    /// Create a new [SubsetSampler].
    pub fn new(set_size: usize) -> Self {
        let rng = Randomizer::new();
        Self { rng, set_size }
    }

    /// Sample a sparse subset of size `subset_size` from the set of indexes from `0` to `set_size`.
    pub fn sample_index_subset(&mut self) -> Vec<usize> {
        let population: Vec<usize> = (0..self.set_size).collect();
        let subset_size = self.rng.next_usize() % self.set_size;
        population
            .into_iter()
            .choose_multiple(&mut self.rng, subset_size)
    }
}
