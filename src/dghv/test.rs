use crate::dghv::{
    Context,
    context::{DGHV_CTX_SMALL, MAX_SECURITY},
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

    // Test for the toy context that the given depht is correct.
    for _ in 0..20 {
        let ctx = DGHV_CTX_SMALL;
        let depth = ctx.max_multiplication_depth(0.0);
        println!("{}", depth);
        let (enc, dec) = ctx.key_gen();
        let mut c1 = enc.encrypt(false);
        let c2 = c1.clone();

        for _ in 0..depth{
            c1 = c1 * c2.clone();
            let res = dec.decrypt(c1.clone());
            assert!(res == false);
        }
    }
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

#[test]
fn memory_footprint_test() {
    // Get the size of a DGHV context.
    let byte_size = DGHV_CTX_SMALL.get_size();
    assert_eq!(byte_size, 0x14);
}
