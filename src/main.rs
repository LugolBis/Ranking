use mylog::logs::init;
use ranking::parser::{api::parse_file, market::market_parser};
use ranking::utils::load_env;
use std::path::PathBuf;

fn main() {
    match init("logs".to_string(), "100mo".to_string(), "1day".to_string()) {
        Ok(_) => {
            if !load_env() {
                println!("Failed to load '.env' file.");
                return;
            }

            let path = std::env::var("MATRIX_PATH").unwrap();
            let m = parse_file(PathBuf::from(path), market_parser);
            match m {
                Ok(matrix) => {
                    println!("{:#?}", matrix);
                }
                Err(mess) => println!("{:?}", mess),
            }
        }
        Err(mess) => println!("{:?}", mess),
    }
}
