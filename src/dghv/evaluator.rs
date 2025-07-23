use rug::{Complete, Integer};

use crate::dghv::Ciphertext;

/// DGHV [Evaluator]. Enables doing operations on [Ciphertext].
#[derive(Debug)]
pub struct Evaluator {
    /// Keys to be able to perform the rescale operation.
    rsk: Integer,
}

impl Evaluator {
    /// Create a new [Evaluator] using the given scale-down key.
    pub fn new(rsk: Integer) -> Self {
        Evaluator { rsk }
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
    pub fn rescale(&self, mut a: Ciphertext) -> Ciphertext {
        self.rescale_inplace(&mut a);
        a
    }

    /// Scale down a given [Ciphertext] to reduce its size.
    pub fn rescale_ref(&self, a: &Ciphertext) -> Ciphertext {
        let mut copy = a.clone();
        self.rescale_inplace(&mut copy);
        copy
    }

    /// Scale down a given [Ciphertext] inplace to reduce its size.
    pub fn rescale_inplace(&self, a: &mut Ciphertext) {
        let raw_a: &mut Integer = a.into();
        *raw_a = raw_a.div_rem_round_ref(&self.rsk).complete().1;
    }

    /// Multiplication of two [Ciphertext] and rescale down the result.
    pub fn mult_and_rescale(&self, a: Ciphertext, b: Ciphertext) -> Ciphertext {
        let mut res = self.mult(a, b);
        self.rescale_inplace(&mut res);
        res
    }

    /// Multiplication of two [Ciphertext] and rescale down the result.
    pub fn mult_and_rescale_ref_right(&self, a: Ciphertext, b: &Ciphertext) -> Ciphertext {
        let mut res = self.mult_ref_right(a, b);
        self.rescale_inplace(&mut res);
        res
    }

    /// Multiplication of two [Ciphertext] and rescale down the result.
    pub fn mult_and_rescale_ref_left(&self, a: &Ciphertext, b: Ciphertext) -> Ciphertext {
        let mut res = self.mult_ref_left(a, b);
        self.rescale_inplace(&mut res);
        res
    }

    /// Multiplication of two [Ciphertext] and rescale down the result.
    pub fn mult_and_rescale_ref_both(&self, a: &Ciphertext, b: &Ciphertext) -> Ciphertext {
        let mut res = self.mult_ref_both(a, b);
        self.rescale_inplace(&mut res);
        res
    }

    /// Multiplication of two [Ciphertext] and rescale down the result inplace.
    pub fn mult_and_rescale_inplace(&self, a: &mut Ciphertext, b: Ciphertext) {
        self.mult_inplace(a, b);
        self.rescale_inplace(a);
    }

    /// Multiplication of two [Ciphertext] and rescale down the result inplace.
    pub fn mult_and_rescale_inplace_ref(&self, a: &mut Ciphertext, b: &Ciphertext) {
        self.mult_inplace_ref(a, b);
        self.rescale_inplace(a);
    }

    /// Get the memory footprint of the [Evaluator].
    pub fn get_size(&self) -> usize {
        let mut size = size_of_val(self);
        size += size_of::<Integer>();
        size += self.rsk.capacity() / (u8::BITS as usize);
        size
    }
}
