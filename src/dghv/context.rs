use crate::{
    dghv::random::new_rand_state,
    dghv::{Decryptor, Encryptor, Evaluator},
};
use rayon::{
    iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use rug::{Complete, Integer, ops::DivRounding};

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
        Some(Context {
            big_noise_width: big_rho,
            noise_width: rho,
            sk_width: eta,
            pk_width: gamma,
            pk_count: tau,
            security: lambda,
        })
    }

    /// Calculate an upper bound on the multiplicative depth
    /// available for the given context parameters. Circuit depth is
    /// estimated via $d \leq \frac{\eta - 4 - \log{|f|}}{\rho\'+2}$.
    /// As $d$ must be smaller than the right term, one is subtracted
    /// from the result to avoid going to close to the limit.
    ///
    /// log_f_norm parameter corresponds with $\log{|f|}$ with $|f|$ the $l_1$
    /// norm of the coefficient vector of $f$, being $f$ the polynomial computed
    /// equivalen to the specific circuit being evaluated. Boolean multiplications
    /// translate into $f(m_1,m_2) = m_1m_2 \rightarrow |f| = \sum\{|1|\}$ on integer algebra
    /// and boolean aditions into $f(m_1,m_2)=m_1+m_2-2m_1m_2 \rightarrow |f| = \sum\{|1|,|1|,|-2|\}$.
    pub fn max_multiplication_depth(&self, log_f_norm: f64) -> u32 {
        let numerator_f64 = self.sk_width as f64 - 4.0 - log_f_norm;
        if numerator_f64 <= 0.0 {
            return 0;
        }
        let denominator_f64 = self.big_noise_width as f64 + 2.0;
        if denominator_f64 == 0.0 {
            return 0;
        }
        // Return one below to make sure the depth is valid.
        ((numerator_f64 / denominator_f64).floor() - 1.0).max(0.0) as u32
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
        let q_bound: Integer = (Integer::from(1) << self.pk_width).div_ceil(secret);
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

    /// Sample a [crate::dghv::Ciphertext] rescaling public key element using the given secret $p$ and index $i$.
    /// Each rescaling element is $x_i\'\leftarrow 2(q_i\'\cdot p + r_i\')$ with
    /// $q_i\'\leftarrow \mathbb{Z}\cap [2^{\gamma+i-1}/p,2^{\gamma+i}/p]$ and $r_i\'=
    /// \mathbb{Z}\cap (-2^\rho, 2^\rho)$.
    fn rescale_key_element_sample(&self, secret: &Integer, index: u32) -> Integer {
        let r_bound = Integer::from(1) << (self.noise_width as u32 + 1);
        let mut r = Integer::from(0);
        while r == 0 {
            r = r_bound.random_below_ref(&mut new_rand_state()).complete()
        }
        r -= r_bound >> 1;
        let shift = self
            .pk_width
            .checked_add(index)
            .unwrap()
            .checked_sub(1)
            .unwrap();
        let q_bound: Integer = (Integer::from(1) << shift).div_ceil(secret);
        let q = q_bound.random_below_ref(&mut new_rand_state()).complete() + q_bound;
        2 * (q * secret + r)
    }

    /// Create a rescaling key using the given secret. A rescaling key is a vector of
    /// $\gamma + 1$ increasingly bigger integers used to reduce [crate::dghv::Ciphertext] size.
    fn rescale_key_sample(&self, secret: &Integer) -> Vec<Integer> {
        let mut rescale_pk: Vec<Integer> = Vec::new();
        rescale_pk.resize(
            self.pk_width.checked_add(1).unwrap() as usize,
            Integer::new(),
        );
        rescale_pk
            .par_iter_mut()
            .zip((0..=self.pk_width).collect::<Vec<_>>())
            .for_each(|x| *x.0 = self.rescale_key_element_sample(secret, x.1));
        rescale_pk
    }

    /// Generate a tuple with an [Encryptor], [Decryptor] and [Evaluator] based on
    /// the parameters of the calling [Context].
    pub fn key_gen(&self) -> (Encryptor, Decryptor, Evaluator) {
        let sk = self.secret_key_sample();
        let pk = self.public_key_sample(&sk);
        let rsk = self.rescale_key_sample(&sk);
        (
            Encryptor::new(pk, self.big_noise_width, self.pk_count),
            Decryptor::new(sk),
            Evaluator::new(rsk, self.pk_width),
        )
    }

    /// Ontain the security parameter $\lambda$ from the
    /// calling [Context].
    pub fn get_security(&self) -> u8 {
        self.security
    }

    /// Get the memory footprint of a given [Context] in bytes.
    pub fn get_size(&self) -> usize {
        std::mem::size_of_val(self)
    }
}
