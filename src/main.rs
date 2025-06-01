use rug::Complete;

pub mod utils;

fn main() {
    for _ in 0..100 {
        let ctx = dghv::DGHV_CTX_MEDIUM;
        let (enc, dec) = ctx.key_gen();
        let ct = enc.encrypt(true);
        let res = dec.decrypt(&ct);
        println!("{res}");
    }
}

pub mod dghv {
    use crate::utils::random::{Randomizer, new_rand_state};
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
        security: 52,
    };

    pub const DGHV_CTX_MEDIUM: Context = Context {
        big_noise_width: 112,
        noise_width: 56,
        sk_width: 2128,
        pk_width: 4251866,
        pk_count: 2110,
        security: 62,
    };

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
            let max_depth = log2_ratio.floor() as u32;
            max_depth
        }

        /// Generate a secret key $p$ from the DGHV context.
        /// This function takes a sample $p\leftarrow(2\mathbb{Z}+1)\cap[2^{\eta-1}, 2^\eta)
        fn secret_key_sample(&self) -> Integer {
            let sk_bound: Integer = Integer::from(1) << (self.sk_width - 1);
            let p: Integer =
                sk_bound.random_below_ref(&mut new_rand_state()).complete() + sk_bound | 0x1;
            p
        }

        /// Get a sample element for a public key from the DGHV context.
        /// Given a secret key $p$ and parameters $\gamma$ and $\rho$. Sample $x=pq+r$
        /// with $q \leftarrow \mathbb{Z}\cap[0, 2^\gamma/p)$ and $r\leftarrow\mathbb{Z}\cap(-2^\rho, 2^\rho)$
        fn public_key_element_sample(&self, secret: &Integer) -> Integer {
            let q_bound: Integer = ((Integer::from(1) << self.pk_width) / secret) + 1;
            let r_bound: Integer = Integer::from(1) << (self.noise_width as u32 + 1);
            let q = q_bound.random_below(&mut new_rand_state());
            let mut r: Integer = Integer::from(0);
            while r == 0 {
                r = r_bound.random_below_ref(&mut new_rand_state()).complete()
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
                let remainder = pk[0].div_rem_round_ref(secret).complete().1;
                if pk[0].find_one(0) == Some(0) && remainder.find_zero(0) == Some(0) {
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
            sum
        }
    }

    #[derive(Debug)]
    pub struct Decryptor {
        sk: Integer,
    }

    impl Decryptor {
        pub fn decrypt(&self, val: &Integer) -> bool {
            let centered_remainder = val.div_rem_round_ref(&self.sk).complete().1;
            let res = centered_remainder.modulo(&Integer::from(2));
            !res.is_zero()
        }
    }

    #[cfg(test)]
    mod test {

        use crate::dghv::*;

        #[test]
        fn context_auto_creation() {
            for i in 0..=MAX_SECURITY {
                assert!(Context::create(i).is_some());
            }
            for i in (MAX_SECURITY + 1..u8::MAX) {
                assert!(Context::create(i).is_none());
            }
        }

        #[test]
        fn max_multiplication_depth() {
            // These values are based on the approximation D < log2(eta / (1 + rho'))
            // and the predefined constants in the Context struct.
            assert_eq!(DGHV_CTX_TINY.max_multiplication_depth(), 4);
            assert_eq!(DGHV_CTX_SMALL.max_multiplication_depth(), 4);
            assert_eq!(DGHV_CTX_MEDIUM.max_multiplication_depth(), 4);
            assert_eq!(DGHV_CTX_LARGE.max_multiplication_depth(), 4);

            // Test a context with potentially 0 depth (if possible to configure)
            let very_noisy_ctx = Context {
                big_noise_width: 1000, // Very large noise
                noise_width: 100,
                sk_width: 100, // Small secret key width
                pk_width: 10000,
                pk_count: 100,
                security: 10,
            };
            // Noise is too large for any multiplication.
            assert_eq!(very_noisy_ctx.max_multiplication_depth(), 0);

            // Test a custom context that might yield a different depth
            let custom_ctx = Context::create(40).unwrap(); // Lambda=40
            assert_eq!(custom_ctx.max_multiplication_depth(), 4);
        }

        #[test]
        fn encryption_decryption() {
            let ctx = DGHV_CTX_SMALL; // Using a predefined small context for testing
            let (enc, dec) = ctx.key_gen();

            // Test encryption and decryption of 'true'
            let ct_true = enc.encrypt(true);
            let decrypted_true = dec.decrypt(&ct_true);
            assert_eq!(decrypted_true, true, "Decryption of true failed!");

            // Test encryption and decryption of 'false'
            let ct_false = enc.encrypt(false);
            let decrypted_false = dec.decrypt(&ct_false);
            assert_eq!(decrypted_false, false, "Decryption of false failed!");
        }

        #[test]
        fn homomorphic_addition() {
            let ctx = DGHV_CTX_SMALL;
            let (enc, dec) = ctx.key_gen();

            // Encrypt true (1) and false (0)
            let ct1 = enc.encrypt(true);  // 1
            let ct2 = enc.encrypt(false); // 0

            // Test 1 + 0 = 1
            let sum_ct = &ct1 + ct2;
            let decrypted_sum = dec.decrypt(&sum_ct);
            assert_eq!(decrypted_sum, true, "Homomorphic addition 1+0 failed!");

            // Test 1 + 1 = 0 (modulo 2)
            let ct3 = enc.encrypt(true); // 1
            let sum_ct2 = &ct1 + ct3;
            let decrypted_sum2 = dec.decrypt(&sum_ct2);
            assert_eq!(decrypted_sum2, false, "Homomorphic addition 1+1 failed!");
        }

        #[test]
        fn homomorphic_multiplication() {
            let ctx = DGHV_CTX_SMALL;
            let (enc, dec) = ctx.key_gen();

            // Encrypt true (1) and false (0)
            let ct1 = enc.encrypt(true);  // 1
            let ct2 = enc.encrypt(false); // 0
            let ct3 = enc.encrypt(true);  // 1

            // Test 1 * 0 = 0
            let mult_ct1 = &ct1 * ct2;
            let decrypted_mult1 = dec.decrypt(&mult_ct1);
            assert_eq!(decrypted_mult1, false, "Homomorphic multiplication 1*0 failed!");

            // Test 1 * 1 = 1
            let mult_ct2 = &ct1 * ct3;
            let decrypted_mult2 = dec.decrypt(&mult_ct2);
            assert_eq!(decrypted_mult2, true, "Homomorphic multiplication 1*1 failed!");
        }
    }
}
