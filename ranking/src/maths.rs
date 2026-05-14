use rand::prelude::*;
use rand::rngs::SmallRng;

/// Compute the norm difference between the two vectors took in input.
pub fn compute_norm(v_new: &[f64], v: &[f64]) -> f64 {
    v_new.iter().zip(v.iter()).map(|(x, y)| (x - y).abs()).sum()
}

/// Generate an uniform vector of the given len.
pub fn uniform_vector(len: usize) -> Vec<f64> {
    vec![1f64 / len as f64; len]
}

#[derive(Debug, Clone)]
/// Wrapper around SmallRng to generate a sequence of random numbers between 0 and 1
pub struct RngSeq {
    rng: SmallRng,
}

impl RngSeq {
    pub fn from(seed: u64) -> Self {
        Self {
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    pub fn next(&mut self) -> f64 {
        self.rng.random()
    }
}
