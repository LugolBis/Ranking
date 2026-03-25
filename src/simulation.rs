use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::parser::{api::parse_file, market::market_parser};

const HEADER: &str = "alpha;percent_of_edges_removed;stationnary_distrib_converge_time";

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Alpha {
    #[serde(default)]
    start: f64,
    #[serde(default)]
    end: f64,
    #[serde(default)]
    step: f64,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Treshold {
    #[serde(default)]
    start: f64,
    #[serde(default)]
    end: f64,
    #[serde(default)]
    step: f64,
}

impl Default for Alpha {
    fn default() -> Self {
        Alpha {
            start: 0.0,
            end: 0.90,
            step: 0.01,
        }
    }
}

impl Default for Treshold {
    fn default() -> Self {
        Treshold {
            start: 0.0,
            end: 0.20,
            step: 0.01,
        }
    }
}

impl Alpha {
    pub fn from(start: f64, end: f64, step: f64) -> Alpha {
        Alpha { start, end, step }
    }
    pub fn start(&self) -> f64 {
        self.start
    }
    pub fn end(&self) -> f64 {
        self.end
    }
    pub fn step(&self) -> f64 {
        self.step
    }
}

impl Treshold {
    pub fn from(start: f64, end: f64, step: f64) -> Treshold {
        Treshold { start, end, step }
    }
    pub fn start(&self) -> f64 {
        self.start
    }
    pub fn end(&self) -> f64 {
        self.end
    }
    pub fn step(&self) -> f64 {
        self.step
    }
}

/// What do we simulate ?<br>
/// We simulate the evolution of stationary distribution convergence time according to the parameters `alpha` and
///  `treshold` who's the percent of edges removed.
///  To minimize I/O cost we load a matrix for a given `treshold` and compute the stationary distribution for all the `alpha`.<br><br>
/// Moreover the simulation follow a geometric sequence `u_n+1` = `matrix(u_n)` - `q` for `n` >= `2`, where `u_0` is the original matrix
/// without any edges removed and `u_1` is the original matrix with exactly (`treshold.step * 100`)% of it's edges removed.
///  With `q` = (`treshold_n` - `treshold_{n-1}`) / (`1.0` - `treshold_{n-1}`).<br><br>
/// These first two terms (`u_0` and `u_1`) are computed by the `simulation.init()` function who only write the matrix `u_1`
///  (required to compute the following terms).
/// After that we compute the `u_2` to `u_n` (where `n` = `treshol_lim` / `treshold.step`) and write each term (matrix).<br><br>
/// That mean the process of removing edges has memory, for example we simulate with `alpha_lim` = `alpha.step` and
///  `treshold_lim` = `0.010`. So we want here to remove `1%` of the edges. So we have the following execution :<br>
/// -> `u_0` -> Original matrix<br>
/// -> `u_1` -> Original matrix - `0.5%` (treshold=0.005) of the edges<br>
/// -> `u_2` -> `u_1` - `q`, with `q` = (`0.01` - `0.005`) / (`1.0` - `0.005`)<br>
/// So `u_2` is the Original matrix - `1%` of edges removed.
pub fn simulation(
    alpha: Alpha,
    treshold: Treshold,
    epsilon: f64,
    matrix_path: &PathBuf,
    output_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let data_path = output_dir.join("results.csv");
    let file = File::create(&data_path)?;

    let mut buffer = BufWriter::new(file);
    writeln!(buffer, "{}", HEADER)?;

    let alpha_steps = (alpha.end / alpha.step) as usize + 1;
    let treshold_steps = (treshold.end / treshold.step) as usize + 1;
    let iterations = alpha_steps * treshold_steps;

    let mut treshold_current = treshold.step * 2f64;
    let mut prev_tresh = treshold.step;
    let mut counter = 0u64;

    init(
        alpha,
        treshold,
        epsilon,
        matrix_path,
        output_dir,
        &mut counter,
        iterations,
        &mut buffer,
    )?;

    while treshold_current <= treshold.end {
        // We just load the previous calculated and writted matrix
        let path = output_dir.join(format!("{}.mtx", prev_tresh));

        // We calculate `q_tresh` the treshold to remove more edges based on the previous treshold : `prev_tresh`
        let q_tresh = (treshold_current - prev_tresh) / (1f64 - prev_tresh);
        let mut matrix = parse_file(&path, market_parser, 0f64)?;

        if q_tresh > 0f64 {
            matrix = matrix.remove_edges(q_tresh)?;
        }

        let mut alpha_current = alpha.start;
        while alpha_current <= alpha.end {
            matrix.set_alpha(alpha_current);

            let (_, steps) = matrix.stationary_distribution(epsilon)?;
            writeln!(buffer, "{};{};{}", alpha_current, treshold_current, steps)?;

            update_counter(&mut counter, iterations);
            alpha_current += alpha.step;
        }

        // We save the computed matrix
        matrix.dump(&output_dir.join(format!("{}.mtx", treshold_current)))?;

        // We update the previous treshold with the current one
        prev_tresh = treshold_current;
        treshold_current += treshold.step;
    }

    println!();
    Ok(())
}

// Compute and save the first iterations.
fn init(
    alpha: Alpha,
    treshold: Treshold,
    epsilon: f64,
    matrix_path: &PathBuf,
    output_dir: &PathBuf,
    counter: &mut u64,
    iterations: usize,
    buffer: &mut BufWriter<File>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut treshold_current = treshold.start;

    while treshold_current <= treshold.step {
        let mut matrix = parse_file(matrix_path, market_parser, 0f64)?;

        if treshold_current > 0f64 {
            matrix = matrix.remove_edges(treshold_current)?;
        }

        let mut alpha_current = alpha.start;
        while alpha_current <= alpha.end {
            matrix.set_alpha(alpha_current);

            let (_, steps) = matrix.stationary_distribution(epsilon)?;
            writeln!(buffer, "{};{};{}", alpha_current, treshold_current, steps)?;

            update_counter(counter, iterations);
            alpha_current += alpha.step;
        }

        if treshold_current > 0f64 {
            let path = output_dir.join(format!("{}.mtx", treshold.step));
            matrix.dump(&path)?;
        }

        treshold_current += treshold.step;
    }
    Ok(())
}

#[inline]
fn update_counter(counter: &mut u64, iterations: usize) {
    *counter += 1;
    print!("\r[Simulation : {}/{}]", counter, iterations);
    let _ = io::stdout().flush();
}
