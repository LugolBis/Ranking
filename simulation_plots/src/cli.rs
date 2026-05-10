use crate::chart::generate;
use std::path::PathBuf;

pub const HELP: &str = r#"
    Usage : simulation_plots <FILE_PATH> <EPSILON> <SEED>
    Example : simulation_plots /path/to/results.csv

    <FILE_PATH> : Absolute file path of the CSV file which contains the simulation results.
    <EPSILON> : The epsilon for the simulation.
    <SEED> : The seed used for the simulation.
"#;

pub fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);

    let data_path = PathBuf::from(args.next().ok_or("Missing data file path in input.")?);

    let epsilon = args
        .next()
        .ok_or("Missing epsilon in input.")?
        .parse::<f64>()
        .or(Err("Failed to parse epsilon.".to_string()))?;

    let seed = args
        .next()
        .ok_or("Missing seed in input.")?
        .parse::<u64>()
        .or(Err("Failed to parse seed.".to_string()))?;

    if let Some(path_dir) = data_path.parent() {
        let chart_path = path_dir.join("chart.png");

        match generate(&data_path, &chart_path, epsilon, seed) {
            Ok(_) => {
                println!("Successfully generate the chart '{:#?}' !", chart_path);
                Ok(())
            }
            Err(err) => Err(format!(
                "Failed to generate the chart due to the error : {}",
                err
            )),
        }
    } else {
        Err(format!(
            "Failed to retrieve the parent folder of the file '{:#?}'",
            data_path
        ))
    }
}
