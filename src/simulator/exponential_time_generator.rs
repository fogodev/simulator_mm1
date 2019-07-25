use rand::prelude::*;

pub struct ExponentialTime {
    random_number_generator: StdRng,
}

impl ExponentialTime {
    pub fn new(seed: u64) -> Self {
        Self {
            random_number_generator: StdRng::seed_from_u64(seed),
        }
    }

    pub fn get(&mut self, lambda: f64) -> f64 {
        -1.0 * (1.0 / lambda) * (self.random_number_generator.gen_range(0.0f64, 1.0f64)).ln()
    }
}
