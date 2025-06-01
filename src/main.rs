use dghv::Context;

pub mod utils;

fn main() {
    

    let mut fail = 0;
    for _ in 0..200 {
        let ctx = dghv::DGHV_CTX_MEDIUM;
        let (enc, dec) = ctx.key_gen();
        let ct = enc.encrypt(true);
        let res = dec.decrypt(&ct);
        println!("{res}");
        if res != true {
            fail += 1;
        }
    }
    println!("{}", fail as f32 / 200.0);
}

pub mod dghv {
    use crate::utils::random::{new_rng_state, Randomizer};
    use rand::seq::IteratorRandom;
    use rayon::{
        iter::{IntoParallelRefMutIterator, ParallelIterator},
        slice::ParallelSliceMut,
    };
    use rug::{Complete, Integer};
    use std::vec::Vec;

    /// Maximum allowed value for $\lambda$ security parameter.
    pub const MAX_SECURITY: u8 = 84;

    /// # DGHV Scheme context.
    /// Store all the parameters used by the scheme.
    #[derive(Debug)]
    pub struct Context {
        /// $\rho\'$ parameter. Secondary noise parameter. Constraint: $\rho\' = \rho + \omega(\log\lambda)$
        big_noise_width: u16,
        /// $\rho$ parameter. Bit-length of the noise. Constraint: $\rho = \omega(\log\lambda)$
        noise_width: u16,
        /// $\eta$ parameter. Bit-length of the secret key. Constraint: $\eta \geq \rho \cdot \Theta(\lambda\log^2\lambda)$
        sk_width: u32,
        /// $\gamma$ parameter. Bit-length of the integers in the public key. Constraint: $\omega(\eta^2\log\lambda)$
        pk_width: u32,
        /// $\tau$ parameter. Number of integers in the public key. Constraint: $\tau \geq \gamma + \omega(\log\lambda)$
        pk_count: u32,
        /// $\lambda$ parameter. General security parameter.
        security: u8,
    }

    pub const DGHV_CTX_TINY: Context = Context {
        big_noise_width: 52,
        noise_width: 26,
        sk_width: 988,
        pk_width: 147456,
        pk_count: 158,
        security: 42,
    };

    pub const DGHV_CTX_SMALL: Context = Context {
        big_noise_width: 82,
        noise_width: 41,
        sk_width: 1558,
        pk_width: 843033,
        pk_count: 572,
        security: 52
    };

    pub const DGHV_CTX_MEDIUM: Context = Context {
        big_noise_width: 112,
        noise_width: 56,
        sk_width: 2128,
        pk_width: 4251866,
        pk_count: 2110,
        security: 62
    };

    pub const DGHV_CTX_LARGE: Context = Context{
        big_noise_width: 142,
        noise_width: 71,
        sk_width: 2698,
        pk_width: 19575950,
        pk_count: 7659,
        security: 72
    };
    
    impl Context {
        /// Create a DGHV context using the recomended configuration.
        /// Takes as input the desired security level.
        pub fn create(security: u8) -> Option<Context> {
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

        /// Generate a secret key $p$ from the DGHV context.
        /// This function takes a sample $p\leftarrow(2\mathbb{Z}+1)\cap[2^{\eta-1}, 2^\eta)
        fn secret_key_sample(&self) -> Integer {
            let sk_bound: Integer = Integer::from(1) << self.sk_width;
            let p: Integer = sk_bound
                .random_below_ref(&mut new_rng_state())
                .complete()
                + sk_bound
                | 0x1;
            p
        }

        /// Get a sample element for a public key from the DGHV context.
        /// Given a secret key $p$ and parameters $\gamma$ and $\rho$. Sample $x=pq+r$
        /// with $q \leftarrow \mathbb{Z}\cap[0, 2^\gamma/p)$ and $r\leftarrow\mathbb{Z}\cap(-2^\rho, 2^\rho)$
        fn public_key_element_sample(&self, secret: &Integer) -> Integer {
            let q_bound: Integer = ((Integer::from(1) << self.pk_width) / secret) + 1;
            let r_bound: Integer = Integer::from(1) << (self.noise_width as u32 + 1);
            let q = q_bound.random_below(&mut new_rng_state());
            let mut r: Integer = Integer::from(0);
            while r == 0 {
              r = r_bound
                .random_below_ref(&mut new_rng_state())
                .complete()  
            }
            r -= r_bound >> 1; 

            let x = (secret * q) + r;
            x
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
                let remainder = pk[0].modulo_ref(secret).complete();
                if pk[0].is_odd() && remainder.is_even() {
                    break;
                }
            }
            pk
        }

        pub fn key_gen(&self) -> (Encryptor, Decryptor) {
            let sk = self.secret_key_sample();
            let pk = self.public_key_sample(&sk);
            (
                Encryptor {
                    pk: pk,
                    big_noise_width: self.big_noise_width,
                    pk_count: self.pk_count,
                },
                Decryptor { sk: sk },
            )
        }
    }

    #[derive(Debug)]
    pub struct Encryptor {
        pk: Vec<Integer>,
        big_noise_width: u16,
        pk_count: u32,
    }

    impl Encryptor {
        pub fn encrypt(&self, val: bool) -> Integer {
            let subset_size = Randomizer::new().random_u32() % self.pk_count;
            let r_bound: Integer = Integer::from(1) << (self.big_noise_width as u32 + 1);
            let mut r: Integer = Integer::from(0);
            while r == 0 {
                r = r_bound.random_below_ref(&mut new_rng_state()).complete()
            }
            r -= (r_bound >> 1);

            let subset = self.pk[1..].iter().choose_multiple(&mut Randomizer::new(), subset_size as usize);

            let mut sum: Integer = subset.into_iter().sum();
            sum *= 2;
            r *= 2;
            sum += r;
            sum += val as u8;
            sum.modulo_mut(&self.pk[0]);

            sum
        }
    }

    #[derive(Debug)]
    pub struct Decryptor {
        sk: Integer,
    }

    impl Decryptor {
        pub fn decrypt(&self, val: &Integer) -> bool {
            let res = val
                .modulo_ref(&self.sk)
                .complete()
                .modulo(&Integer::from(2));
            !res.is_zero()
        }
    }

    #[cfg(test)]
    mod test {

        use super::{Context, MAX_SECURITY};

        #[test]
        fn context_auto_creation() {
            for i in 0..=MAX_SECURITY {
                assert!(Context::create(i).is_some());
            }
            for i in (MAX_SECURITY + 1..u8::MAX) {
                assert!(Context::create(i).is_none());
            }
        }
    }
}
