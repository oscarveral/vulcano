const REPETITIONS: usize = 30;

#[test]
fn encrypt_decrypt() {
    for _ in 0..REPETITIONS {
        // Use the tiny parameters for performance.
        let mut ctx = crate::dghv::context::Context::new(crate::dghv::parameters::TINY);

        // Generate scheme elements.
        let (mut enc, dec, _) = ctx.key_gen();

        // Test with true encryption.
        let true_enc = enc.encrypt(true);
        let dec_val_true = dec.decrypt(true_enc);
        assert_eq!(true, dec_val_true);

        // Test with false encryption.
        let false_enc = enc.encrypt(false);
        let dec_val_false = dec.decrypt(false_enc);
        assert_eq!(false, dec_val_false);
    }
}

#[test]
fn addition() {
    for _ in 0..REPETITIONS {
        // Use the tiny parameters for performance.
        let mut ctx = crate::dghv::context::Context::new(crate::dghv::parameters::TINY);

        // Generate scheme elements.
        let (mut enc, dec, eval) = ctx.key_gen();

        // Calculation values.
        let true_enc = enc.encrypt(true);
        let false_enc = enc.encrypt(false);

        // 0 + 0 = 0.
        let res1 = eval.add(false_enc.clone(), false_enc.clone());
        let dec1 = dec.decrypt(res1);
        assert_eq!(false, dec1);

        // 0 + 1 = 1.
        let res2 = eval.add(false_enc.clone(), true_enc.clone());
        let dec2 = dec.decrypt(res2);
        assert_eq!(true, dec2);

        // 1 + 0 = 1.
        let res3 = eval.add(true_enc.clone(), false_enc.clone());
        let dec3 = dec.decrypt(res3);
        assert_eq!(true, dec3);

        // 1 + 1 = 0.
        let res4 = eval.add(true_enc.clone(), true_enc.clone());
        let dec4 = dec.decrypt(res4);
        assert_eq!(false, dec4);
    }
}

#[test]
fn multiplication() {
    for _ in 0..REPETITIONS {
        // Use the tiny parameters for performance.
        let mut ctx = crate::dghv::context::Context::new(crate::dghv::parameters::TINY);

        // Generate scheme elements.
        let (mut enc, dec, eval) = ctx.key_gen();

        // Calculation values.
        let true_enc = enc.encrypt(true);
        let false_enc = enc.encrypt(false);

        // 0 * 0 = 0.
        let res1 = eval.mul(false_enc.clone(), false_enc.clone());
        let dec1 = dec.decrypt(res1);
        assert_eq!(false, dec1);

        // 0 * 1 = 0.
        let res2 = eval.mul(false_enc.clone(), true_enc.clone());
        let dec2 = dec.decrypt(res2);
        assert_eq!(false, dec2);

        // 1 * 0 = 0.
        let res3 = eval.mul(true_enc.clone(), false_enc.clone());
        let dec3 = dec.decrypt(res3);
        assert_eq!(false, dec3);

        // 1 * 1 = 1.
        let res4 = eval.mul(true_enc.clone(), true_enc.clone());
        let dec4 = dec.decrypt(res4);
        assert_eq!(true, dec4);
    }
}

#[test]
fn max_depth() {
    for _ in 0..REPETITIONS {
        // Use the tiny parameters for performance.
        let mut ctx = crate::dghv::context::Context::new(crate::dghv::parameters::TINY);
        let max_depth = crate::dghv::parameters::TINY.max_multiplication_depth(0.0);

        // Generate scheme elements.
        let (mut enc, dec, eval) = ctx.key_gen();

        // Calculation values.
        let true_enc = enc.encrypt(true);
        let false_enc = enc.encrypt(false);
        let mut res_true = true_enc.clone();
        let mut res_false = false_enc.clone();

        // Consume all multiplicative levels.
        for _ in 0..max_depth {
            res_true = eval.mul(true_enc.clone(), res_true.clone());
            res_false = eval.mul(false_enc.clone(), res_false.clone());
        }

        // Assert results are correct.
        let dec_true = dec.decrypt(res_true);
        let dec_false = dec.decrypt(res_false);
        assert_eq!(true, dec_true);
        assert_eq!(false, dec_false);
    }
}

#[test]
fn downsize() {
    for _ in 0..REPETITIONS {
        // Use the tiny parameters for performance.
        let mut ctx = crate::dghv::context::Context::new(crate::dghv::parameters::TINY);
        let max_depth = crate::dghv::parameters::TINY.max_multiplication_depth(0.0);

        // Generate scheme elements.
        let (mut enc, dec, eval) = ctx.key_gen();

        // Calculation values.
        let true_enc = enc.encrypt(true);
        let false_enc = enc.encrypt(false);
        let mut res_true = true_enc.clone();
        let mut res_false = false_enc.clone();

        // Consume all multiplicative levels.
        for _ in 0..max_depth {
            res_true = eval.mul(true_enc.clone(), res_true);
            res_true = eval.downsize(res_true);
            res_false = eval.mul(false_enc.clone(), res_false);
            res_false = eval.downsize(res_false);
        }

        // Assert results are correct.
        let dec_true = dec.decrypt(res_true);
        let dec_false = dec.decrypt(res_false);
        assert_eq!(true, dec_true);
        assert_eq!(false, dec_false);
    }
    for _ in 0..REPETITIONS {
        // Use the tiny parameters for performance.
        let mut ctx = crate::dghv::context::Context::new(crate::dghv::parameters::TINY);
        let max_depth = crate::dghv::parameters::TINY.max_multiplication_depth(0.0);

        // Generate scheme elements.
        let (mut enc, dec, eval) = ctx.key_gen();

        // Calculation values.
        let true_enc = enc.encrypt(true);
        let false_enc = enc.encrypt(false);
        let mut res_true = true_enc.clone();
        let mut res_false = false_enc.clone();

        // Consume all multiplicative levels minus one and make a sum.
        for _ in 0..(max_depth - 1) {
            res_true = eval.mul(true_enc.clone(), res_true);
            res_true = eval.downsize(res_true);
            res_false = eval.mul(false_enc.clone(), res_false);
            res_false = eval.downsize(res_false);
        }

        res_true = eval.add(res_true.clone(), true_enc.clone());
        res_false = eval.add(res_false.clone(), true_enc.clone());

        // Assert results are correct.
        let dec_true = dec.decrypt(res_true);
        let dec_false = dec.decrypt(res_false);
        assert_eq!(false, dec_true);
        assert_eq!(true, dec_false);
    }
    for _ in 0..REPETITIONS {
        // Use the tiny parameters for performance.
        let mut ctx = crate::dghv::context::Context::new(crate::dghv::parameters::TINY);

        // Generate scheme elements.
        let (mut enc, dec, eval) = ctx.key_gen();

        // Calculation values.
        let true_enc = enc.encrypt(true);
        let false_enc = enc.encrypt(false);
        let mut res_true = true_enc.clone();
        let mut res_false = false_enc.clone();

        // Downsize originals.
        let mut dec_true = dec.decrypt(res_true);
        let mut dec_false = dec.decrypt(res_false);
        assert_eq!(true, dec_true);
        assert_eq!(false, dec_false);

        // Downsize some only sums.
        res_true = eval.add(true_enc.clone(), true_enc.clone());
        res_true = eval.add(res_true, false_enc.clone());
        res_false = eval.add(false_enc.clone(), false_enc.clone());
        res_false = eval.add(res_false, true_enc.clone());
        dec_true = dec.decrypt(res_true);
        dec_false = dec.decrypt(res_false);
        assert_eq!(false, dec_true);
        assert_eq!(true, dec_false);
    }
}
