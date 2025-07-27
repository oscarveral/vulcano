use rug::{Integer, ops::DivRounding};
use std::str::FromStr;

/// Divides `dividend` by `divisor` using **"rounding half up"** logic and obtain the quotient.
/// The function is implemented using the formula `floor((2 * dividend + divisor) / (2 * divisor))`.
/// By definition, this function computes `$q(n, d) = \lfloor n / d \rceil$` with `$\lfloor z \rceil$$`
/// defined as `$\mathbb{Z}\cap(z-\frac{1}{2}, z+\frac{1}{2}]$`.
///
/// # Panics
/// Panics if `divisor` is `0`.
pub fn quotient(dividend: &Integer, divisor: &Integer) -> Integer {
    if divisor.is_zero() {
        panic!("Division by zero!");
    }

    let numerator = Integer::from(2) * dividend + divisor;
    let denominator = Integer::from(2) * divisor;

    numerator.div_floor(denominator)
}

/// Divides `dividend` by `divisor` using **"rounding half up"** logic and obtain the remainder.
/// Result is consistent with [quotient] function. Remainder is computed as
/// `$r(n, d) = n - q(n, d) * d$`, with `$q$` being the quotient function. Remainder value
/// falls in the `$(-p/2, p/2]$` range.
///
/// # Panics
/// Panics if `divisor` is `0`.
pub fn remainder(dividend: &Integer, divisor: &Integer) -> Integer {
    if divisor.is_zero() {
        panic!("Division by zero!");
    }

    let q = quotient(dividend, divisor);
    dividend - q * divisor
}

#[test]
fn quotient_samples() {
    /// Helper function for creating Integer from i32 for brevity in tests
    fn i(val: i32) -> rug::Integer {
        rug::Integer::from(val)
    }

    assert_eq!(quotient(&i(10), &i(2)), i(5));
    assert_eq!(quotient(&i(100), &i(10)), i(10));
    assert_eq!(quotient(&i(-10), &i(2)), i(-5));
    assert_eq!(quotient(&i(-100), &i(10)), i(-10));
    assert_eq!(quotient(&i(10), &i(-2)), i(-5));
    assert_eq!(quotient(&i(-10), &i(-2)), i(5));

    assert_eq!(quotient(&i(5), &i(2)), i(3)); // 2.5 -> 3
    assert_eq!(quotient(&i(7), &i(2)), i(4)); // 3.5 -> 4
    assert_eq!(quotient(&i(99), &i(2)), i(50)); // 49.5 -> 50
    assert_eq!(quotient(&i(-5), &i(-2)), i(3)); // 2.5 -> 3
    assert_eq!(quotient(&i(-5), &i(2)), i(-2)); // -2.5 -> -2
    assert_eq!(quotient(&i(-7), &i(2)), i(-3)); // -3.5 -> -3
    assert_eq!(quotient(&i(-99), &i(2)), i(-49)); // -49.5 -> -49
    assert_eq!(quotient(&i(5), &i(-2)), i(-2)); // -2.5 -> -2

    assert_eq!(quotient(&i(8), &i(3)), i(3)); // 2.66... -> 3
    assert_eq!(quotient(&i(11), &i(3)), i(4)); // 3.66... -> 4
    assert_eq!(quotient(&i(-8), &i(-3)), i(3)); // 2.66... -> 3
    assert_eq!(quotient(&i(-8), &i(3)), i(-3)); // -2.66... -> -3
    assert_eq!(quotient(&i(-11), &i(3)), i(-4)); // -3.66... -> -4
    assert_eq!(quotient(&i(8), &i(-3)), i(-3)); // -2.66... -> -3

    assert_eq!(quotient(&i(7), &i(3)), i(2)); // 2.33... -> 2
    assert_eq!(quotient(&i(10), &i(3)), i(3)); // 3.33... -> 3
    assert_eq!(quotient(&i(-7), &i(-3)), i(2)); // 2.33... -> 2
    assert_eq!(quotient(&i(-7), &i(3)), i(-2)); // -2.33... -> -2
    assert_eq!(quotient(&i(-10), &i(3)), i(-3)); // -3.33... -> -3
    assert_eq!(quotient(&i(7), &i(-3)), i(-2)); // -2.33... -> -2

    assert_eq!(quotient(&i(0), &i(5)), i(0));
    assert_eq!(quotient(&i(0), &i(-5)), i(0));
    assert_eq!(quotient(&i(0), &i(1)), i(0));
    assert_eq!(quotient(&i(0), &i(1_000_000)), i(0));

    let num = rug::Integer::from_str("123456789012345678901234567890123456789").unwrap();
    let den = rug::Integer::from_str("7").unwrap();
    let expected = rug::Integer::from_str("17636684144620811271604938270017636684").unwrap();
    assert_eq!(quotient(&num, &den), expected);

    let num_neg = rug::Integer::from_str("-987654321098765432109876543210987654321").unwrap();
    let expected_neg = rug::Integer::from_str("-329218107032921810703292181070329218107").unwrap();
    assert_eq!(quotient(&num_neg, &i(3)), expected_neg);

    let num_neg2 = rug::Integer::from_str("-123456789012345678901234567890123456789").unwrap();
    let den_neg2 = rug::Integer::from_str("-7").unwrap();
    assert_eq!(quotient(&num_neg2, &den_neg2), expected);

    let num_neg_half_up =
        rug::Integer::from_str("-1000000000000000000000000000000000000005").unwrap();
    let den_half_up = rug::Integer::from_str("10").unwrap();
    let expected_neg_half_up =
        rug::Integer::from_str("-100000000000000000000000000000000000000").unwrap();
    assert_eq!(
        quotient(&num_neg_half_up, &den_half_up),
        expected_neg_half_up
    );

    let num_pos_half_up =
        rug::Integer::from_str("1000000000000000000000000000000000000005").unwrap();
    let expected_pos_half_up =
        rug::Integer::from_str("100000000000000000000000000000000000001").unwrap();
    assert_eq!(
        quotient(&num_pos_half_up, &den_half_up),
        expected_pos_half_up
    );
}

#[test]
fn remainder_samples() {
    /// Helper function for creating Integer from i32 for brevity in tests
    fn i(val: i32) -> rug::Integer {
        rug::Integer::from(val)
    }

    assert_eq!(remainder(&i(10), &i(2)), i(0));
    assert_eq!(remainder(&i(100), &i(10)), i(0));
    assert_eq!(remainder(&i(-10), &i(2)), i(0));
    assert_eq!(remainder(&i(-100), &i(10)), i(0));
    assert_eq!(remainder(&i(10), &i(-2)), i(0));
    assert_eq!(remainder(&i(-10), &i(-2)), i(0));

    assert_eq!(remainder(&i(5), &i(2)), i(-1)); // 5 = 3 * 2 + (-1)
    assert_eq!(remainder(&i(7), &i(2)), i(-1)); // 7 = 4 * 2 + (-1)
    assert_eq!(remainder(&i(99), &i(2)), i(-1)); // 99 = 50 * 2 + (-1)
    assert_eq!(remainder(&i(-5), &i(-2)), i(1)); // -5 = 3 * -2 + 1
    assert_eq!(remainder(&i(-5), &i(2)), i(-1)); // -5 = -2 * 2 + (-1)
    assert_eq!(remainder(&i(-7), &i(2)), i(-1)); // -7 = -3 * 2 + (-1)
    assert_eq!(remainder(&i(-99), &i(2)), i(-1)); // -99 = -49 * 2 + (-1)
    assert_eq!(remainder(&i(5), &i(-2)), i(1)); // 5 = -2 * -2 + 1

    assert_eq!(remainder(&i(8), &i(3)), i(-1)); // 8 = 3 * 3 + (-1)
    assert_eq!(remainder(&i(11), &i(3)), i(-1)); // 11 = 4 * 3 + (-1)
    assert_eq!(remainder(&i(-8), &i(-3)), i(1)); // -8 = 3 * -3 + 1
    assert_eq!(remainder(&i(-8), &i(3)), i(1)); // -8 = -3 * 3 + 1
    assert_eq!(remainder(&i(-11), &i(3)), i(1)); // -11 = -4 * 3 + 1
    assert_eq!(remainder(&i(8), &i(-3)), i(-1)); // 8 = -3 * -3 + (-1)

    assert_eq!(remainder(&i(7), &i(3)), i(1)); // 7 = 2 * 3 + 1
    assert_eq!(remainder(&i(10), &i(3)), i(1)); // 10 = 3 * 3 + 1
    assert_eq!(remainder(&i(-7), &i(-3)), i(-1)); // -7 = 2 * -3 + (-1)
    assert_eq!(remainder(&i(-7), &i(3)), i(-1)); // -7 = -2 * 3 + (-1)
    assert_eq!(remainder(&i(-10), &i(3)), i(-1)); // -10 = -3 * 3 + (-1)
    assert_eq!(remainder(&i(7), &i(-3)), i(1)); // 7 = -2 * -3 + 1

    assert_eq!(remainder(&i(0), &i(5)), i(0));
    assert_eq!(remainder(&i(0), &i(-5)), i(0));
    assert_eq!(remainder(&i(0), &i(1)), i(0));
    assert_eq!(remainder(&i(0), &i(1_000_000)), i(0));

    let num = rug::Integer::from_str("123456789012345678901234567890123456789").unwrap();
    let den = rug::Integer::from_str("7").unwrap();
    assert_eq!(remainder(&num, &den), i(1));

    let num_neg = rug::Integer::from_str("-987654321098765432109876543210987654321").unwrap();
    assert_eq!(remainder(&num_neg, &i(3)), i(0));

    let num_neg2 = rug::Integer::from_str("-123456789012345678901234567890123456789").unwrap();
    let den_neg2 = rug::Integer::from_str("-7").unwrap();
    assert_eq!(remainder(&num_neg2, &den_neg2), i(-1));

    let num_neg_half_up =
        rug::Integer::from_str("-1000000000000000000000000000000000000005").unwrap();
    let den_half_up = rug::Integer::from_str("10").unwrap();
    assert_eq!(remainder(&num_neg_half_up, &den_half_up), i(-5));

    let num_pos_half_up =
        rug::Integer::from_str("1000000000000000000000000000000000000005").unwrap();
    assert_eq!(remainder(&num_pos_half_up, &den_half_up), i(-5));
}
