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

#[test]
fn test_rng() {
    let mut rng1 = RngSeq::from(1);
    let mut rng2 = RngSeq::from(1);
    let mut rng3 = RngSeq::from(3);

    for _ in 0..100_000 {
        assert_eq!(rng1.next(), rng2.next());
        assert_ne!(rng1.next(), rng3.next());
        assert_ne!(rng2.next(), rng3.next());
    }
}
