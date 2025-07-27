use rug::Integer;

/// A ciphertext of the DGHV scheme.
#[derive(Clone)]
pub struct Ciphertext(
    /// Internally, a [Ciphertext] is only an integer.
    Integer,
);

impl From<Integer> for Ciphertext {
    fn from(value: Integer) -> Self {
        Ciphertext(value)
    }
}

impl From<Ciphertext> for Integer {
    fn from(value: Ciphertext) -> Self {
        value.0
    }
}
