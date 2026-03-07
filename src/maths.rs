pub fn compute_norm(v_new: &[f64], v: &[f64]) -> f64 {
    v_new.iter().zip(v.iter()).map(|(x, y)| (x - y).abs()).sum()
}

pub fn uniform_vector(len: usize) -> Vec<f64> {
    vec![1f64 / len as f64; len]
}
