use rand::{rngs::StdRng, RngCore, SeedableRng};
use rug::rand::{RandGen, RandState};

pub fn new_rand_state() -> RandState<'static> {
        let random = Box::new(Randomizer::new());
        RandState::new_custom_boxed(random)
}

pub struct Randomizer {
    rng: StdRng,
}

impl Randomizer {
    pub fn new() -> Randomizer {
        let rng = StdRng::from_os_rng();
        Randomizer { rng: rng }
    }

    pub fn random_usize(&mut self) -> usize {
        self.rng.next_u64() as usize
    }

    pub fn random_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    pub fn random_u64(&mut self) -> u64 {
        self.rng.next_u64()
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