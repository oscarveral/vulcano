use rug::{Complete, Integer};

use crate::dghv::Ciphertext;

/// An [Evaluator] that enables the operation on DGHV [Ciphertext].
#[derive(Clone, Debug)]
pub struct Evaluator {
    /// Vector of keys to be able to perform the downscale operation.
    rsk: Vec<Integer>,
    /// $\gamma$ parameter. Bit-length of the integers in the public key. Constraint: $\omega(\eta^2\log\lambda)$.
    pk_width: u32,
}

impl Evaluator {
    /// Create a new [Evaluator] using the given scale-down keys and $\gamma$ parameter.
    pub fn new(rsk: Vec<Integer>, pk_width: u32) -> Self {
        Evaluator { rsk, pk_width }
    }

    /// Addition of two [Ciphertext].
    pub fn add(&self, a: Ciphertext, b: Ciphertext) -> Ciphertext {
        let a_raw: Integer = a.into();
        let b_raw: Integer = b.into();
        (a_raw + b_raw).into()
    }

    /// Addition of two [Ciphertext].
    pub fn add_ref_right(&self, a: Ciphertext, b: &Ciphertext) -> Ciphertext {
        let a_raw: Integer = a.into();
        let b_raw: &Integer = b.into();
        (a_raw + b_raw).into()
    }

    /// Addition of two [Ciphertext].
    pub fn add_ref_left(&self, a: &Ciphertext, b: Ciphertext) -> Ciphertext {
        let a_raw: &Integer = a.into();
        let b_raw: Integer = b.into();
        (b_raw + a_raw).into()
    }

    /// Addition of two [Ciphertext].
    pub fn add_ref_both(&self, a: &Ciphertext, b: &Ciphertext) -> Ciphertext {
        let a_raw: &Integer = a.into();
        let b_raw: &Integer = b.into();
        (a_raw + b_raw).complete().into()
    }

    /// Inplace addition of two [Ciphertext].
    pub fn add_inplace(&self, a: &mut Ciphertext, b: Ciphertext) {
        let a_raw: &mut Integer = a.into();
        let b_raw: Integer = b.into();
        *a_raw += b_raw;
    }

    /// Inplace addition of two [Ciphertext].
    pub fn add_inplace_ref(&self, a: &mut Ciphertext, b: &Ciphertext) {
        let a_raw: &mut Integer = a.into();
        let b_raw: &Integer = b.into();
        *a_raw += b_raw;
    }

    /// Multiplication of two [Ciphertext].
    pub fn mult(&self, a: Ciphertext, b: Ciphertext) -> Ciphertext {
        let a_raw: Integer = a.into();
        let b_raw: Integer = b.into();
        (a_raw * b_raw).into()
    }

    /// Multiplication of two [Ciphertext].
    pub fn mult_ref_right(&self, a: Ciphertext, b: &Ciphertext) -> Ciphertext {
        let a_raw: Integer = a.into();
        let b_raw: &Integer = b.into();
        (a_raw * b_raw).into()
    }

    /// Multiplication of two [Ciphertext].
    pub fn mult_ref_left(&self, a: &Ciphertext, b: Ciphertext) -> Ciphertext {
        let a_raw: &Integer = a.into();
        let b_raw: Integer = b.into();
        (b_raw * a_raw).into()
    }

    /// Multiplication of two [Ciphertext].
    pub fn mult_ref_both(&self, a: &Ciphertext, b: &Ciphertext) -> Ciphertext {
        let a_raw: &Integer = a.into();
        let b_raw: &Integer = b.into();
        (a_raw * b_raw).complete().into()
    }

    /// Inplace multiplication of two [Ciphertext].
    pub fn mult_inplace(&self, a: &mut Ciphertext, b: Ciphertext) {
        let a_raw: &mut Integer = a.into();
        let b_raw: Integer = b.into();
        *a_raw *= b_raw;
    }

    /// Inplace multiplication of two [Ciphertext].
    pub fn mult_inplace_ref(&self, a: &mut Ciphertext, b: &Ciphertext) {
        let a_raw: &mut Integer = a.into();
        let b_raw: &Integer = b.into();
        *a_raw *= b_raw;
    }

    /// Scale down a given [Ciphertext] to reduce its size.
    pub fn scale_down(&self, a: &mut Ciphertext) {
        let bound: Integer = Integer::from(1) << self.pk_width;
        let raw_a: &mut Integer = a.into();
        if *raw_a > bound {
            for i in (0..=self.pk_width as usize).rev() {
                *raw_a = (*raw_a).div_rem_round_ref(&self.rsk[i]).complete().1;
            }
        }
    }

    /// Get the memory footprint of the [Evaluator].
    pub fn get_size(&self) -> usize {
        let mut size = size_of_val(self);
        for i in &self.rsk {
            size += size_of::<Integer>();
            size += i.capacity() / (u8::BITS as usize);
        }
        size
    }
}
