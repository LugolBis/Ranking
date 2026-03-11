use ranking::parser::{api::parse_file, market::market_parser};
use ranking::utils::load_env;
use std::path::PathBuf;

const ALPHA: f64 = 0.85;
const EPSILON: f64 = 1e-6;

fn main() {
    if !load_env() {
        println!("Failed to load '.env' file.");
        return;
    }

    let path = std::env::var("MATRIX_PATH").unwrap();
    let m = parse_file(PathBuf::from(path), market_parser, ALPHA);
    match m {
        Ok(matrix) => {
            if let Ok((vec, steps)) = matrix.stationary_distribution(EPSILON) {
                println!("Sum of distribution = {:?}", vec.iter().sum::<f64>());
                println!("Step : {}", steps);
            };
        }
        Err(mess) => println!("{:?}", mess),
    }
}
