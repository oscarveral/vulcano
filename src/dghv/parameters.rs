/// DGHV [Parameters].
/// Store all the parameters used by the scheme.
#[derive(Debug)]
pub struct Parameters {
    /// $\rho\'$ parameter. Secondary noise parameter. Constraint: $\rho\' = \rho + \omega(\log\lambda)$.
    pub rho_prime: u16,
    /// $\rho$ parameter. Bit-length of the noise. Constraint: $\rho = \omega(\log\lambda)$.
    pub rho: u16,
    /// $\eta$ parameter. Bit-length of the generator key. Constraint: $\eta \geq \rho \cdot \Theta(\lambda\log^2\lambda)$.
    pub eta: u32,
    /// $\gamma$ parameter. Bit-length of the integers in the public key. Constraint: $\omega(\eta^2\log\lambda)$.
    pub gamma: u32,
    /// $\tau$ parameter. Number of integers in the public key. Constraint: $\tau \geq \gamma + \omega(\log\lambda)$.
    pub tau: u32,
    /// $\lambda$ parameter. General security parameter.
    pub lambda: u8,
    /// $\kappa$ parameter. Bit precision of elements of the bootstrapping key. Constraint: $\kappa = \gamma + 2$.
    pub kappa: u32,
    /// $\theta$ parameter. Size of the sparse subset of bootstrapping keys that compose the public key. Constraint $\theta = \lambda$.
    pub theta: u8,
    /// $\Theta$ parameter. Number of samples on the bootstrapping key. Constraint: $\Theta = \omega(\kappa\log\lambda)$.
    pub theta_big: u32,
}

/// Standard DGHV toy [Parameters] set.
pub const TINY: Parameters = Parameters {
    rho_prime: 52,
    rho: 26,
    eta: 988,
    gamma: 147456,
    tau: 158,
    lambda: 42,
    kappa: 147458,
    theta: 42,
    theta_big: 150,
};

/// Standard DGHV [Parameters] set that yield smaller keys.
pub const SMALL: Parameters = Parameters {
    rho_prime: 82,
    rho: 41,
    eta: 1558,
    gamma: 843033,
    tau: 572,
    lambda: 52,
    kappa: 843035,
    theta: 52,
    theta_big: 555,
};

/// Standard DGHV secure [Parameters] for medium-sized keys.
pub const MEDIUM: Parameters = Parameters {
    rho_prime: 112,
    rho: 56,
    eta: 2128,
    gamma: 4251866,
    tau: 2110,
    lambda: 62,
    kappa: 4251868,
    theta: 62,
    theta_big: 2070,
};

/// Standard DGHV secure [Parameters] that yield large keys.
pub const LARGE: Parameters = Parameters {
    rho_prime: 142,
    rho: 71,
    eta: 2698,
    gamma: 19575950,
    tau: 7659,
    lambda: 72,
    kappa: 19575952,
    theta: 72,
    theta_big: 7965,
};

impl Parameters {
    /// Calculate an upper bound on the multiplicative depth
    /// available for the given context parameters. Circuit depth is
    /// estimated via $d \leq \frac{\eta - 4 - \log{|f|}}{\rho\'+2}$.
    /// As $d$ must be smaller than the right term, one is subtracted
    /// from the result to avoid going too close to the limit.
    ///
    /// The log_f_norm parameter corresponds with $\log{|f|}$ with $|f|$ the $l_1$
    /// norm of the coefficient vector of $f$, being $f$ the polynomial computed
    /// equivalent to the specific circuit being evaluated. Boolean multiplications
    /// translate into $f(m_1,m_2) = m_1m_2 \rightarrow |f| = \sum\lbrace|1|\rbrace$ on integer algebra
    /// and boolean additions into $f(m_1,m_2)=m_1+m_2-2m_1m_2 \rightarrow |f| = \sum\lbrace|1|,|1|,|-2|\rbrace$.
    pub fn max_multiplication_depth(&self, log_f_norm: f64) -> u32 {
        let numerator_f64 = self.eta as f64 - 4.0 - log_f_norm;
        if numerator_f64 <= 0.0 {
            return 0;
        }
        let denominator_f64 = self.rho_prime as f64 + 2.0;
        if denominator_f64 == 0.0 {
            return 0;
        }
        // Return the one below to make sure the depth is valid.
        ((numerator_f64 / denominator_f64).floor() - 1.0).max(0.0) as u32
    }
}
