use crate::chart::generate;
use std::path::PathBuf;

const HELP: &str = r#"
    Usage : simulation_plots <FILE_PATH> <EPSILON>
    Example : simulation_plots /path/to/results.csv

    <FILE_PATH> : Absolute file path of the CSV file which contains the simulation results.
    <EPSILON> : The epsilon of the simulation.
"#;

pub fn main() {
    let mut args = std::env::args().skip(1);

    if let (Some(path_str), Some(epsilon_str)) = (args.next(), args.next()) {
        let data_path = PathBuf::from(path_str);

        match epsilon_str.parse::<f64>() {
            Ok(epsilon) => {
                if let Some(path_dir) = data_path.parent() {
                    let chart_path = path_dir.join("chart.png");

                    match generate(&data_path, &chart_path, epsilon) {
                        Ok(_) => println!("Successfully generate the chart '{:#?}' !", chart_path),
                        Err(err) => {
                            println!("Failed to generate the chart due to the error : {}", err)
                        }
                    }
                } else {
                    println!(
                        "Failed to retrieve the parent folder of the file '{:#?}'",
                        data_path
                    );
                }
            }
            Err(err) => {
                println!("Failed to parse epsilon '{}' due to '{}'", epsilon_str, err);
            }
        }
    } else {
        println!("{}", HELP);
    }
}
