use crate::traits::{primitive::Unsigned, size::Fixed};

use crate::natural::Natural;

impl<T: Unsigned + Fixed> From<&str> for Natural<T> {
    fn from(value: &str) -> Self {
        let mut num = Self::default();
        let radix: u32 = 10;
        for c in value.chars() {
            if let Some(digit) = c.to_digit(radix) {
                num *= T::exact_from(radix);
                num += T::exact_from(digit);
            }
        }
        num
    }
}
