use ranking::parser::{api::parse_file, market::market_parser};
use ranking::utils::{dump_matrix, load_env, parse_args};
use std::path::PathBuf;

/// CSC computations.
fn compute_csc(alpha: f64, epsilon: f64, treashold: f64, path: PathBuf) {
    match parse_file(PathBuf::from(path), market_parser, alpha, treashold) {
        Ok(matrix) => match matrix.stationary_distribution(epsilon) {
            Ok((vec, steps)) => {
                println!("Sum of distribution = {}", vec.iter().sum::<f64>());
                println!("Step : {}", steps);

                if let Err(e) = dump_matrix(matrix, PathBuf::from("my_matrix.mtx")) {
                    eprintln!("{}", e)
                }
            }
            Err(e) => eprintln!("{}", e),
        },
        Err(e) => eprintln!("{}", e),
    }
}

fn main() {
    match parse_args() {
        Ok((alpha, epsilon, treshold, opt_path)) => {
            if let Some(path) = opt_path {
                compute_csc(alpha, epsilon, path);
            } else {
                if !load_env() {
                    eprintln!(
                        "Failed to load `.env` file, who need to be at the root of the project."
                    );
                } else {
                    match std::env::var("MATRIX_PATH") {
                        Ok(path) => {
                            compute_csc(alpha, epsilon, PathBuf::from(path));
                        }
                        Err(e) => eprintln!(
                            "Failed to get the environment variable `MATRIX_PATH` : {}",
                            e
                        ),
                    }
                }
            }
        }
        Err(e) => eprintln!("{}", e),
    }
}
