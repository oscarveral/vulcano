use crate::{
    dghv::random::new_rand_state,
    dghv::{Decryptor, Encryptor, Evaluator},
};
use rayon::{
    iter::{IntoParallelRefMutIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use rug::{Complete, Integer, ops::DivRounding};

/// DGHV [Context].
/// Store all the parameters used by the scheme.
#[derive(Debug)]
pub struct Context {
    /// $\rho\'$ parameter. Secondary noise parameter. Constraint: $\rho\' = \rho + \omega(\log\lambda)$.
    rho_prime: u16,
    /// $\rho$ parameter. Bit-length of the noise. Constraint: $\rho = \omega(\log\lambda)$.
    rho: u16,
    /// $\eta$ parameter. Bit-length of the generator key. Constraint: $\eta \geq \rho \cdot \Theta(\lambda\log^2\lambda)$.
    eta: u32,
    /// $\gamma$ parameter. Bit-length of the integers in the public key. Constraint: $\omega(\eta^2\log\lambda)$.
    gamma: u32,
    /// $\tau$ parameter. Number of integers in the public key. Constraint: $\tau \geq \gamma + \omega(\log\lambda)$.
    tau: u32,
    /// $\lambda$ parameter. General security parameter.
    lambda: u8,
    /// $\kappa$ parameter. Bit precision of elements of the bootstrapping key. Constraint: $\kappa = \gamma + 2$.
    kappa: u32,
    /// $\theta$ parameter. Size of the sparse subset of bootstrapping keys that compose the public key. Constraint $\theta = \lambda$.
    theta: u8,
    /// $\Theta$ parameter. Number of samples on the bootstrapping key. Constraint: $\Theta = \omega(\kappa\log\lambda)$.
    theta_big: u32,
}

/// Standard DGHV context with toy parameters.
pub const CONTEXT_TINY: Context = Context {
    rho_prime: 52,
    rho: 26,
    eta: 988,
    gamma: 147456,
    tau: 158,
    lambda: 42,
    kappa: 147458,
    theta: 42,
    theta_big: 150,
};

/// Standard DGHV context with parameters that yield smaller keys.
pub const CONTEXT_SMALL: Context = Context {
    rho_prime: 82,
    rho: 41,
    eta: 1558,
    gamma: 843033,
    tau: 572,
    lambda: 52,
    kappa: 843035,
    theta: 52,
    theta_big: 555,
};

/// Standard DGHV context with secure parameters for medium-sized keys.
pub const CONTEXT_MEDIUM: Context = Context {
    rho_prime: 112,
    rho: 56,
    eta: 2128,
    gamma: 4251866,
    tau: 2110,
    lambda: 62,
    kappa: 4251868,
    theta: 62,
    theta_big: 2070,
};

/// Standard DGHV context with secure parameters that yield large keys.
pub const CONTEXT_LARGE: Context = Context {
    rho_prime: 142,
    rho: 71,
    eta: 2698,
    gamma: 19575950,
    tau: 7659,
    lambda: 72,
    kappa: 19575952,
    theta: 72,
    theta_big: 7965,
};

impl Context {
    /// Calculate an upper bound on the multiplicative depth
    /// available for the given context parameters. Circuit depth is
    /// estimated via $d \leq \frac{\eta - 4 - \log{|f|}}{\rho\'+2}$.
    /// As $d$ must be smaller than the right term, one is subtracted
    /// from the result to avoid going too close to the limit.
    ///
    /// The log_f_norm parameter corresponds with $\log{|f|}$ with $|f|$ the $l_1$
    /// norm of the coefficient vector of $f$, being $f$ the polynomial computed
    /// equivalent to the specific circuit being evaluated. Boolean multiplications
    /// translate into $f(m_1,m_2) = m_1m_2 \rightarrow |f| = \sum\lbrace|1|\rbrace$ on integer algebra
    /// and boolean additions into $f(m_1,m_2)=m_1+m_2-2m_1m_2 \rightarrow |f| = \sum\lbrace|1|,|1|,|-2|\rbrace$.
    pub fn max_multiplication_depth(&self, log_f_norm: f64) -> u32 {
        let numerator_f64 = self.eta as f64 - 4.0 - log_f_norm;
        if numerator_f64 <= 0.0 {
            return 0;
        }
        let denominator_f64 = self.rho_prime as f64 + 2.0;
        if denominator_f64 == 0.0 {
            return 0;
        }
        // Return the one below to make sure the depth is valid.
        ((numerator_f64 / denominator_f64).floor() - 1.0).max(0.0) as u32
    }

    /// Generate a generator key $p$ from the DGHV context.
    /// This function takes a sample $p\leftarrow(2\mathbb{Z}+1)\cap[2^{\eta-1}, 2^\eta)$.
    fn generator_key_sample(&self) -> Integer {
        let sk_bound: Integer = Integer::from(1) << (self.eta.checked_sub(1).unwrap());
        let p: Integer =
            (sk_bound.random_below_ref(&mut new_rand_state()).complete() + sk_bound) | 0x1;
        p
    }

    /// Get a sample element for a public key from the DGHV context.
    /// Given a generator key $p$ and parameters $\gamma$ and $\rho$. Sample $x=pq+r$
    /// with $q \leftarrow \mathbb{Z}\cap[0, 2^\gamma/p)$ and $r\leftarrow\mathbb{Z}\cap(-2^\rho, 2^\rho)$.
    fn public_key_element_sample(&self, secret: &Integer) -> Integer {
        let q_bound: Integer = (Integer::from(1) << self.gamma).div_ceil(secret);
        let r_bound: Integer = Integer::from(1) << (self.rho as u32 + 1);
        let q = q_bound.random_below(&mut new_rand_state());
        let mut r: Integer = Integer::from(0);
        while r == 0 {
            r = r_bound.random_below_ref(&mut new_rand_state()).complete()
        }
        r -= r_bound >> 1;

        (secret * q) + r
    }

    /// Get the first element of the public key.
    /// Given a generator key $p$ and parameter $\gamma$. Sample $x_0=pq$ with
    /// $q \leftarrow \mathbb{Z}\cap[0, 2^\gamma/p)$.
    fn rescale_key_sample(&self, secret: &Integer) -> Integer {
        let q_bound: Integer = (Integer::from(1) << self.gamma).div_ceil(secret);
        let q = q_bound.random_below(&mut new_rand_state());
        secret * q
    }

    /// Generate a public key $pk$ from a DGHV context and a given generator $p$.
    /// $pk$ is a collection of $\tau + 1$ elements sampled the distribution specified on the
    /// [Context::public_key_element_sample](Context::public_key_element_sample) except $pk_0 that
    /// satisfies $pk_0$ is the largest one, $pk_0$ is odd and $pk_0$ is an exact multiple of $p$.
    /// $pk_0$ will be used also as the rescaling key.
    fn public_key_sample(&self, secret: &Integer) -> (Vec<Integer>, Integer) {
        let mut pk: Vec<Integer> = Vec::new();
        pk.resize((self.tau as usize).checked_add(1).unwrap(), Integer::new());
        loop {
            pk[1..].par_iter_mut().for_each(|x| {
                *x = self.public_key_element_sample(secret);
            });

            pk[0] = self.rescale_key_sample(secret);

            pk[1..].par_sort();
            pk[1..].reverse();

            // Assert our chosen $pk_0$ is the bigger element of the public key.
            if pk[0] < pk[1] {
                continue;
            }

            if pk[0].find_one(0) == Some(0) {
                break;
            }
        }
        let rsk = pk[0].clone();
        (pk, rsk)
    }

    /// Generate a tuple with an [Encryptor], [Decryptor] and [Evaluator] based on
    /// the parameters of the calling [Context].
    pub fn key_gen(&self) -> (Encryptor, Decryptor, Evaluator) {
        let g = self.generator_key_sample();
        let (pk, rsk) = self.public_key_sample(&g);
        (
            Encryptor::new(pk, self.rho_prime, self.tau),
            Decryptor::new(g),
            Evaluator::new(rsk),
        )
    }

    /// Get the security parameter $\lambda$ from the
    /// calling [Context].
    pub fn get_security(&self) -> u8 {
        self.lambda
    }

    /// Get the memory footprint of a given [Context] in bytes.
    pub fn get_size(&self) -> usize {
        size_of_val(self)
    }
}
