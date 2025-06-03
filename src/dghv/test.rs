use crate::dghv::{
    Context,
    context::{DGHV_CTX_LARGE, DGHV_CTX_MEDIUM, DGHV_CTX_SMALL, DGHV_CTX_TINY, MAX_SECURITY},
};

#[test]
fn context_auto_creation() {
    for i in 0..=MAX_SECURITY {
        assert!(Context::create_with_derivation(i).is_some());
    }
    for i in (MAX_SECURITY + 1)..u8::MAX {
        assert!(Context::create_with_derivation(i).is_none());
    }
}

#[test]
fn max_multiplication_depth() {
    // Test default provided DGHV context depth.
    assert_eq!(DGHV_CTX_TINY.max_multiplication_depth(), 4);
    assert_eq!(DGHV_CTX_SMALL.max_multiplication_depth(), 4);
    assert_eq!(DGHV_CTX_MEDIUM.max_multiplication_depth(), 4);
    assert_eq!(DGHV_CTX_LARGE.max_multiplication_depth(), 4);

    // Test a context with potentially 0 depth.
    let very_noisy_ctx_build = Context::create_with_params(10, 100, 1000, 100, 10000, 100);
    assert!(very_noisy_ctx_build.is_some());
    let very_noisy_ctx = very_noisy_ctx_build.unwrap();
    assert_eq!(very_noisy_ctx.max_multiplication_depth(), 0);
}

#[test]
fn encryption_decryption() {
    // Use a small sized context for testing.
    let ctx = DGHV_CTX_SMALL;
    let (enc, dec) = ctx.key_gen();

    // Test encryption and decryption of 'true'.
    let ct_true = enc.encrypt(true);
    let decrypted_true = dec.decrypt(ct_true);
    assert_eq!(decrypted_true, true, "Decryption of true failed!");

    // Test encryption and decryption of 'false'.
    let ct_false = enc.encrypt(false);
    let decrypted_false = dec.decrypt(ct_false);
    assert_eq!(decrypted_false, false, "Decryption of false failed!");
}

#[test]
fn homomorphic_addition() {
    // Use a small sized context for testing.
    let ctx = DGHV_CTX_SMALL;
    let (enc, dec) = ctx.key_gen();

    // Encrypt true (1) and false (0).
    let ct1 = enc.encrypt(true);
    let ct2 = enc.encrypt(false);

    // Test 1 + 0 = 1.
    let sum_ct = ct1.clone() + ct2.clone();
    let decrypted_sum = dec.decrypt(sum_ct);
    assert_eq!(decrypted_sum, true, "Homomorphic addition 1+0 failed!");

    // Test 1 + 1 = 0.
    let sum_ct2 = ct2.clone() + ct2;
    let decrypted_sum2 = dec.decrypt(sum_ct2);
    assert_eq!(decrypted_sum2, false, "Homomorphic addition 1+1 failed!");

    // Test 0 + 0 = 0.
    let sum_ct3 = ct1.clone() + ct1;
    let decrypted_sum3 = dec.decrypt(sum_ct3);
    assert_eq!(decrypted_sum3, false, "Homomorphic addition 0+0 failed!");
}

#[test]
fn homomorphic_multiplication() {
    // Use a small sized context for testing.
    let ctx = DGHV_CTX_SMALL;
    let (enc, dec) = ctx.key_gen();

    // Encrypt true (1) and false (0).
    let ct1 = enc.encrypt(true);
    let ct2 = enc.encrypt(false);

    // Test 1 * 0 = 0.
    let mult_ct1 = ct1.clone() * ct2.clone();
    let decrypted_mult1 = dec.decrypt(mult_ct1);
    assert_eq!(
        decrypted_mult1, false,
        "Homomorphic multiplication 1*0 failed!"
    );

    // Test 1 * 1 = 1.
    let mult_ct2 = ct1.clone() * ct1;
    let decrypted_mult2 = dec.decrypt(mult_ct2);
    assert_eq!(
        decrypted_mult2, true,
        "Homomorphic multiplication 1*1 failed!"
    );

    // Test 0 * 0 = 0.
    let mult_ct3 = ct2.clone() * ct2;
    let decrypted_mult3 = dec.decrypt(mult_ct3);
    assert_eq!(
        decrypted_mult3, false,
        "Homomorphic multiplication 0*0 failed!"
    );
}
