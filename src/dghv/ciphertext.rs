use rug::Integer;

/// DGHV [Ciphertext].
/// Store private data.
#[derive(Clone, Debug)]
pub struct Ciphertext {
    /// Internally, a ciphertext is only an integer.
    val: Integer,
    /// Metadata needed for decryption and bootstrapping.
    meta: Option<Vec<Integer>>,
}

impl From<Integer> for Ciphertext {
    fn from(value: Integer) -> Self {
        Ciphertext {
            val: value,
            meta: None,
        }
    }
}

impl From<Ciphertext> for Integer {
    fn from(value: Ciphertext) -> Self {
        value.val
    }
}

impl<'a> From<&'a Ciphertext> for &'a Integer {
    fn from(value: &'a Ciphertext) -> Self {
        &value.val
    }
}

impl<'a> From<&'a mut Ciphertext> for &'a mut Integer {
    fn from(value: &'a mut Ciphertext) -> Self {
        &mut value.val
    }
}

impl Ciphertext {
    /// Set the metadata for the [Ciphertext].
    pub fn set_meta(&mut self, meta: Vec<Integer>) {
        self.meta = Some(meta);
    }

    /// Get a reference for the metadata of the [Ciphertext].
    pub fn get_meta(&self) -> &Option<Vec<Integer>> {
        &self.meta
    }

    /// Get the memory footprint of the [Ciphertext].
    pub fn get_size(&self) -> usize {
        let mut size = size_of_val(self);
        size += self.val.capacity() / (u8::BITS as usize);
        if self.meta.is_some() {
            if let Some(vec) = &self.meta {
                for i in vec {
                    size += i.capacity() / (u8::BITS as usize);
                }
            }
        }
        size
    }
}
