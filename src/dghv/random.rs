use rand::{RngCore, SeedableRng, rngs::StdRng};
use rug::rand::{RandGen, RandState};

/// Box a [Randomizer] with in a [RandState] to be used as a
/// random number provider in [rug].
pub fn new_rand_state() -> RandState<'static> {
    let random = Box::new(Randomizer::new());
    RandState::new_custom_boxed(random)
}

/// Random number generator for the DGHV scheme.
pub struct Randomizer {
    /// Internal random number provider.
    rng: StdRng,
}

impl Randomizer {
    /// Create a new random number generator.
    pub fn new() -> Randomizer {
        let rng = StdRng::from_os_rng();
        Randomizer { rng }
    }
}

impl Default for Randomizer {
    fn default() -> Self {
        Self::new()
    }
}

impl RandGen for Randomizer {
    fn r#gen(&mut self) -> u32 {
        self.rng.next_u32()
    }
}

impl RngCore for Randomizer {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    fn fill_bytes(&mut self, dst: &mut [u8]) {
        self.rng.fill_bytes(dst);
    }
}
