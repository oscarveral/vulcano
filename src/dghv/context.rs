use crate::{
    dghv::{Decryptor, Encryptor},
    utils::random::new_rand_state,
};
use rayon::{
    iter::{IntoParallelRefMutIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use rug::{Complete, Integer};

/// Maximum implementation allowed value for $\lambda$ security parameter.
pub const MAX_SECURITY: u8 = 84;

/// DGHV Scheme [Context].
/// Store all the parameters used by the scheme.
#[derive(Debug)]
pub struct Context {
    /// $\rho\'$ parameter. Secondary noise parameter. Constraint: $\rho\' = \rho + \omega(\log\lambda)$.
    big_noise_width: u16,
    /// $\rho$ parameter. Bit-length of the noise. Constraint: $\rho = \omega(\log\lambda)$.
    noise_width: u16,
    /// $\eta$ parameter. Bit-length of the secret key. Constraint: $\eta \geq \rho \cdot \Theta(\lambda\log^2\lambda)$.
    sk_width: u32,
    /// $\gamma$ parameter. Bit-length of the integers in the public key. Constraint: $\omega(\eta^2\log\lambda)$.
    pk_width: u32,
    /// $\tau$ parameter. Number of integers in the public key. Constraint: $\tau \geq \gamma + \omega(\log\lambda)$.
    pk_count: u32,
    /// $\lambda$ parameter. General security parameter.
    security: u8,
}

/// Standard DGHV context with toy parameters.
pub const DGHV_CTX_TINY: Context = Context {
    big_noise_width: 52,
    noise_width: 26,
    sk_width: 988,
    pk_width: 147456,
    pk_count: 158,
    security: 42,
};

/// Standard DGHV context with parameters that yield smaller keys.
pub const DGHV_CTX_SMALL: Context = Context {
    big_noise_width: 82,
    noise_width: 41,
    sk_width: 1558,
    pk_width: 843033,
    pk_count: 572,
    security: 52,
};

/// Standard DGHV context with secure parameters for medium-sized keys.
pub const DGHV_CTX_MEDIUM: Context = Context {
    big_noise_width: 112,
    noise_width: 56,
    sk_width: 2128,
    pk_width: 4251866,
    pk_count: 2110,
    security: 62,
};

/// Standard DGHV context with secure parameters that yield large keys.
pub const DGHV_CTX_LARGE: Context = Context {
    big_noise_width: 142,
    noise_width: 71,
    sk_width: 2698,
    pk_width: 19575950,
    pk_count: 7659,
    security: 72,
};

impl Context {
    /// Create a DGHV context using deriving all parameters.
    /// Takes as input the desired security level used to derive the rest.
    pub fn create_with_derivation(security: u8) -> Option<Context> {
        // Bigger security values may produce param values bigger than 32-bit max value.
        if security > MAX_SECURITY {
            return None;
        }

        let lambda: u8 = security;
        let rho: u16 = lambda as u16;
        let big_rho: u16 = 2 * (lambda as u16);
        let eta: u32 = (lambda as u32) * (lambda as u32) + (lambda as u32);
        let gamma: u32 = (lambda as u32) * eta * eta;
        let tau: u32 = (lambda as u32) + gamma;

        Some(Context {
            security: lambda,
            sk_width: eta,
            pk_width: gamma,
            pk_count: tau,
            noise_width: rho,
            big_noise_width: big_rho,
        })
    }

    /// Create a DGHV context specifying all the parameters.
    /// Be carefull with the parameters used as it may result in an insecure scheme.
    pub fn create_with_params(
        lambda: u8,
        rho: u16,
        big_rho: u16,
        eta: u32,
        gamma: u32,
        tau: u32,
    ) -> Option<Context> {
        if lambda > MAX_SECURITY {
            return None;
        }

        Some(Context {
            big_noise_width: big_rho,
            noise_width: rho,
            sk_width: eta,
            pk_width: gamma,
            pk_count: tau,
            security: lambda,
        })
    }

    /// Estimates the maximum multiplication depth supported by the context.
    /// This is a heuristic based on the approximate noise growth in DGHV.
    /// Noise roughly squares its bit-length with each multiplication.
    /// The formula used is $d < \log_2(\eta / (1 + \rho\'))$.
    pub fn max_multiplication_depth(&self) -> u32 {
        let eta = self.sk_width as f64;
        let big_rho_prime = self.big_noise_width as f64;
        let numerator = eta;
        let denominator = 1.0 + big_rho_prime;
        let ratio = numerator / denominator;
        // If ratio is 1 or less, $log_2(ratio)$ would be 0 or negative.
        // This means even a fresh ciphertext's noise might be too large,
        // or no multiplications are supported.
        if ratio <= 1.0 {
            return 0;
        }
        let log2_ratio = ratio.log2();
        // The maximum depth is the floor of this value
        // (since $d$ must be an integer and $d < value$)
        log2_ratio.floor() as u32
    }

    /// Generate a secret key $p$ from the DGHV context.
    /// This function takes a sample $p\leftarrow(2\mathbb{Z}+1)\cap[2^{\eta-1}, 2^\eta).
    fn secret_key_sample(&self) -> Integer {
        let sk_bound: Integer = Integer::from(1) << (self.sk_width - 1);
        let p: Integer =
            (sk_bound.random_below_ref(&mut new_rand_state()).complete() + sk_bound) | 0x1;
        p
    }

    /// Get a sample element for a public key from the DGHV context.
    /// Given a secret key $p$ and parameters $\gamma$ and $\rho$. Sample $x=pq+r$
    /// with $q \leftarrow \mathbb{Z}\cap[0, 2^\gamma/p)$ and $r\leftarrow\mathbb{Z}\cap(-2^\rho, 2^\rho)$.
    fn public_key_element_sample(&self, secret: &Integer) -> Integer {
        let q_bound: Integer = ((Integer::from(1) << self.pk_width) / secret) + 1;
        let r_bound: Integer = Integer::from(1) << (self.noise_width as u32 + 1);
        let q = q_bound.random_below(&mut new_rand_state());
        let mut r: Integer = Integer::from(0);
        while r == 0 {
            r = r_bound.random_below_ref(&mut new_rand_state()).complete()
        }
        r -= r_bound >> 1;

        (secret * q) + r
    }

    /// Generate a public key $pk$ from a DGHV context and a given secret $p$.
    /// $pk$ is a collection of $\tau + 1$ elements sampled the distribution specified on the
    /// [Context::public_key_element_sample](Context::public_key_element_sample) that satisfies
    /// that $pk_0$ is the largest one, $pk_0$ is odd, and $pk_0\;\text{mod}\;p$ is even.
    fn public_key_sample(&self, secret: &Integer) -> Vec<Integer> {
        let mut pk: Vec<Integer> = Vec::new();
        pk.resize(self.pk_count as usize + 1, Integer::new());
        loop {
            pk.par_iter_mut().for_each(|x| {
                *x = self.public_key_element_sample(secret);
            });
            pk.par_sort();
            pk.reverse();
            let remainder = pk[0].div_rem_round_ref(secret).complete().1;
            if pk[0].find_one(0) == Some(0) && remainder.find_zero(0) == Some(0) {
                break;
            }
        }
        pk
    }

    /// Generate a pair of with an [Encryptor] and [Decryptor] based on
    /// the parameters of the calling [Context].
    pub fn key_gen(&self) -> (Encryptor, Decryptor) {
        let sk = self.secret_key_sample();
        let pk = self.public_key_sample(&sk);
        (
            Encryptor::new(pk, self.big_noise_width, self.pk_count),
            Decryptor::new(sk),
        )
    }
}
