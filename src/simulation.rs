use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
};

use crate::{
    matrix::CSC,
    parser::{api::parse_file, market::market_parser},
};

const HEADER: &str = "alpha;arc_removed;stationnary_distrib_converge_time";
const ALPHA_STEP: f64 = 0.005f64;
const TRESHOLD_STEP: f64 = 0.005f64;

pub fn simulation(
    alpha_lim: f64,
    epsilon: f64,
    treshold_lim: f64,
    matrix_path: &PathBuf,
    output_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(&output_dir.join("results.csv"))?;

    let mut buffer = BufWriter::new(file);
    writeln!(buffer, "{}", HEADER)?;

    let mut first_iteration = true;

    let alpha_steps = (alpha_lim / ALPHA_STEP) as usize + 1;
    let treshold_steps = (treshold_lim / TRESHOLD_STEP) as usize + 1;
    let iterations = alpha_steps * treshold_steps;
    let mut counter = 1u64;

    for alpha_c in 0..alpha_steps {
        let alpha = alpha_c as f64 * ALPHA_STEP;
        let mut prev_proba = 0f64;

        for treshold_c in 0..treshold_steps {
            print!("\r[Simulation : {}/{}]", counter, iterations);
            io::stdout().flush()?;

            let treshold = treshold_c as f64 * TRESHOLD_STEP;
            let matrix: CSC;
            if treshold == 0f64 {
                // We just load the matrix without removing any edge
                matrix = parse_file(matrix_path, market_parser, alpha, treshold)?;
                prev_proba = 0f64;
            } else if first_iteration {
                if prev_proba > 0f64 {
                    // We calculate `q` the treshold to remove more arcs based on the previous treshold : `prev_proba`
                    let q = (treshold - prev_proba) / (1f64 - prev_proba);
                    let path = output_dir.join(format!("{}.mtx", prev_proba));

                    matrix = parse_file(&path, market_parser, alpha, q)?;
                    prev_proba = treshold;
                } else {
                    matrix = parse_file(matrix_path, market_parser, alpha, treshold)?;
                    prev_proba = treshold;
                }
            } else {
                // We just load the already calculated and writted matrix
                let path = output_dir.join(format!("{}.mtx", treshold));
                matrix = parse_file(&path, market_parser, alpha, 0f64)?;
            }

            let (_, steps) = matrix.stationary_distribution(epsilon)?;
            writeln!(buffer, "{};{};{}", alpha, treshold, steps)?;

            if first_iteration && treshold > 0f64 {
                matrix.dump(&output_dir.join(format!("{}.mtx", treshold)))?;
            }

            counter += 1;
        }

        first_iteration = false;
    }

    Ok(())
}
