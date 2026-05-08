use std::hash::{BuildHasher, Hasher, RandomState};

const EXPONENT_MASK: u64 = 1023 << 52;

/// Compute the norm difference between the two vectors took in input.
pub fn compute_norm(v_new: &[f64], v: &[f64]) -> f64 {
    v_new.iter().zip(v.iter()).map(|(x, y)| (x - y).abs()).sum()
}

/// Generate an uniform vector of the given len.
pub fn uniform_vector(len: usize) -> Vec<f64> {
    vec![1f64 / len as f64; len]
}

/// Generate a random value between 0.0 and 1.0
pub fn random() -> f64 {
    f64::from_bits(EXPONENT_MASK | RandomState::new().build_hasher().finish() >> 12) - 1.0
}
