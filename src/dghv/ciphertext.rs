use rug::Integer;

/// DGHV [Ciphertext].
/// Store private data.
#[derive(Clone, Debug)]
pub struct Ciphertext(
    /// Internally a ciphertext is only an integer.
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

impl<'a> From<&'a Ciphertext> for &'a Integer {
    fn from(value: &'a Ciphertext) -> Self {
        &value.0
    }
}

impl<'a> From<&'a mut Ciphertext> for &'a mut Integer {
    fn from(value: &'a mut Ciphertext) -> Self {
        &mut value.0
    }
}

impl Ciphertext {
    /// Obtain the memory footprint of the [Ciphertext].
    pub fn get_size(&self) -> usize {
        let size = std::mem::size_of_val(self);
        size + (self.0.capacity() / (u8::BITS as usize))
    }
}
