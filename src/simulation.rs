use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
};

use crate::parser::{api::parse_file, market::market_parser};

const HEADER: &str = "alpha;arc_removed;stationnary_distrib_converge_time";
const ALPHA_STEP: f64 = 0.01f64;
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

    let alpha_steps = (alpha_lim / ALPHA_STEP) as usize + 1;
    let treshold_steps = (treshold_lim / TRESHOLD_STEP) as usize + 1;
    let iterations = alpha_steps * treshold_steps;

    let mut prev_tresh = TRESHOLD_STEP;
    let mut counter = 0u64;

    init(
        alpha_steps,
        epsilon,
        matrix_path,
        output_dir,
        &mut counter,
        iterations,
        &mut buffer,
    )?;

    for treshold_c in 2..treshold_steps {
        let treshold = (treshold_c as f64) * TRESHOLD_STEP;

        // We just load the previous calculated and writted matrix
        let path = output_dir.join(format!("{}.mtx", prev_tresh));

        // We calculate `q_tresh` the treshold to remove more edges based on the previous treshold : `prev_tresh`
        let q_tresh = (treshold - prev_tresh) / (1f64 - prev_tresh);
        let mut matrix = parse_file(&path, market_parser, 0f64, q_tresh)?;

        for alpha_c in 0..alpha_steps {
            let alpha = alpha_c as f64 * ALPHA_STEP;
            matrix.set_alpha(alpha);

            let (_, steps) = matrix.stationary_distribution(epsilon)?;
            writeln!(buffer, "{};{};{}", alpha, treshold, steps)?;

            update_counter(&mut counter, iterations);
        }

        // We save the computed matrix
        matrix.dump(&output_dir.join(format!("{}.mtx", treshold)))?;

        // We update the previous treshold with the current one
        prev_tresh = treshold;
    }

    Ok(())
}

// Compute and save the first two iterations
fn init(
    alpha_steps: usize,
    epsilon: f64,
    matrix_path: &PathBuf,
    output_dir: &PathBuf,
    counter: &mut u64,
    iterations: usize,
    buffer: &mut BufWriter<File>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut start = false;

    for treshold_c in 0usize..2usize {
        let treshold = treshold_c as f64 * TRESHOLD_STEP;
        let mut matrix = parse_file(matrix_path, market_parser, 0f64, treshold)?;

        for alpha_c in 0..alpha_steps {
            let alpha = alpha_c as f64 * ALPHA_STEP;
            matrix.set_alpha(alpha);

            let (_, steps) = matrix.stationary_distribution(epsilon)?;
            writeln!(buffer, "{};{};{}", alpha, treshold, steps)?;

            update_counter(counter, iterations);
        }

        if !start {
            let path = output_dir.join(format!("{}.mtx", TRESHOLD_STEP));
            matrix.dump(&path)?;
        }

        start = false;
    }
    Ok(())
}

#[inline]
fn update_counter(counter: &mut u64, iterations: usize) {
    *counter += 1;
    print!("\r[Simulation : {}/{}]", counter, iterations);
    let _ = io::stdout().flush();
}
