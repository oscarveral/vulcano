use crate::dghv::context::CONTEXT_TINY;

#[test]
fn encryption_decryption() {
    // Use a small-sized context for testing.
    let ctx = CONTEXT_TINY;
    let (enc, dec, _) = ctx.key_gen();

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
    // Use a small-sized context for testing.
    let ctx = CONTEXT_TINY;
    let (enc, dec, eval) = ctx.key_gen();

    // Encrypt true (1) and false (0).
    let ct1 = enc.encrypt(true);
    let ct2 = enc.encrypt(false);

    // Test 1 + 0 = 1.
    let sum_ct = eval.add_ref_both(&ct1, &ct2);
    let decrypted_sum = dec.decrypt(sum_ct);
    assert_eq!(decrypted_sum, true, "Homomorphic addition 1+0 failed!");

    // Test 1 + 1 = 0.
    let sum_ct2 = eval.add(ct1.clone(), ct1);
    let decrypted_sum2 = dec.decrypt(sum_ct2);
    assert_eq!(decrypted_sum2, false, "Homomorphic addition 1+1 failed!");

    // Test 0 + 0 = 0.
    let sum_ct3 = eval.add(ct2.clone(), ct2);
    let decrypted_sum3 = dec.decrypt(sum_ct3);
    assert_eq!(decrypted_sum3, false, "Homomorphic addition 0+0 failed!");
}

#[test]
fn homomorphic_multiplication() {
    // Use a small-sized context for testing.
    let ctx = CONTEXT_TINY;
    let (enc, dec, eval) = ctx.key_gen();

    // Encrypt true (1) and false (0).
    let ct1 = enc.encrypt(true);
    let ct2 = enc.encrypt(false);

    // Test 1 * 0 = 0.
    let mult_ct1 = eval.mult_ref_both(&ct1, &ct2);
    let decrypted_mult1 = dec.decrypt(mult_ct1);
    assert_eq!(
        decrypted_mult1, false,
        "Homomorphic multiplication 1*0 failed!"
    );

    // Test 1 * 1 = 1.
    let mult_ct2 = eval.mult(ct1.clone(), ct1);
    let decrypted_mult2 = dec.decrypt(mult_ct2);
    assert_eq!(
        decrypted_mult2, true,
        "Homomorphic multiplication 1*1 failed!"
    );

    // Test 0 * 0 = 0.
    let mult_ct3 = eval.mult(ct2.clone(), ct2);
    let decrypted_mult3 = dec.decrypt(mult_ct3);
    assert_eq!(
        decrypted_mult3, false,
        "Homomorphic multiplication 0*0 failed!"
    );
}

#[test]
fn scale_down() {
    // Use a small-sized context for testing.
    let ctx = CONTEXT_TINY;
    let (enc, _, eval) = ctx.key_gen();
    // Encrypt sample values.
    let mut ct1 = enc.encrypt(true);
    let ct2 = enc.encrypt(true);
    // Grow a big ciphertext a scale down as needed.
    let mut size = ct1.get_size();
    for _ in 0..40 {
        eval.mult_inplace_ref(&mut ct1, &ct2);
        size = match size < ct1.get_size() {
            true => ct1.get_size(),
            false => size,
        };
        eval.scale_down(&mut ct1);
    }
    // See if max reached size is bigger than the scaled down one.
    let size_after = ct1.get_size();
    assert!(size_after <= size);
}

#[test]
fn max_multiplication_depth() {
    // Test for the toy context that the given depth is correct.
    for _ in 0..5 {
        let ctx = CONTEXT_TINY;
        let depth = ctx.max_multiplication_depth(0.0);
        let (enc, dec, eval) = ctx.key_gen();
        let mut c1 = enc.encrypt(false);
        let c2 = c1.clone();

        for _ in 0..depth {
            eval.mult_inplace_ref(&mut c1, &c2);
            let res = dec.decrypt(c1.clone());
            assert_eq!(res, false, "Expected a false value on the assertion!");
        }
    }
}
