use crate::chart::generate;
use crate::errors::CLIErr;
use crate::parser::{api::parse_file, market::market_parser};
use crate::simulation::simulation;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct CLI {
    alpha: f64,
    epsilon: f64,
    treshold: f64,
    matrix_path: PathBuf,
    output_dir: PathBuf,
    help: bool,
    simulate: bool,
}

const HELP: &str = r#"
    Usage : ranking <ARGS>
    Example : ranking --alpha 0.85 --epsilon 1e-6 --treshold 0.01 --env_path /path/to/your/.env

    ARGS:
        --alpha, -a (Required) : The alpha used in Page Rank algorithm.
        --espilon, -e (Required) : To manage the precision in stationarry distribution.
        --treshold, -t (Required) : The percent of edges to be removed (needs to be in the interval [0.0 ; 1.0])
        --matrix-path, -m : Matrix file path.
        --output-dir, -o : Directory used for output files.
        --env-path, -env : Environment file path who contains the variables.
        --help, -h : Display this message.
        --simulate, -s : Simulate the evolution of the stationary distribution convergence according to `alpha` and `treshold`.
"#;

impl CLI {
    /// Parse the CLI arguments.
    fn try_new() -> Result<CLI, CLIErr> {
        let mut alpha: Option<f64> = None;
        let mut epsilon: Option<f64> = None;
        let mut treshold: Option<f64> = None;
        let mut matrix_path: Option<PathBuf> = None;
        let mut output_dir: Option<PathBuf> = None;
        let mut env_path: Option<PathBuf> = None;
        let mut help = false;
        let mut simulate = false;

        let mut args = std::env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--alpha" | "-a" => {
                    let value = args
                        .next()
                        .ok_or(CLIErr::Alpha("Missing value for --alpha".into()))?;

                    alpha = Some(
                        value
                            .parse::<f64>()
                            .map_err(|e| CLIErr::Alpha(format!("Failed to parse alpha: {}", e)))?,
                    );
                }

                "--epsilon" | "-e" => {
                    let value = args
                        .next()
                        .ok_or(CLIErr::Epsilon("Missing value for --epsilon".into()))?;

                    epsilon =
                        Some(value.parse::<f64>().map_err(|e| {
                            CLIErr::Epsilon(format!("Failed to parse epsilon: {}", e))
                        })?);
                }

                "--treshold" | "-t" => {
                    let value = args
                        .next()
                        .ok_or(CLIErr::Treshold("Missing value for --treshold".into()))?;

                    treshold = Some(value.parse::<f64>().map_err(|e| {
                        CLIErr::Treshold(format!("Failed to parse treshold: {}", e))
                    })?);
                }

                "--matrix-path" | "-m" => {
                    let value = args
                        .next()
                        .ok_or(CLIErr::MatrixPath("Missing value for --matrix-path".into()))?;

                    matrix_path = Some(PathBuf::from(value));
                }

                "--output-dir" | "-o" => {
                    let value = args
                        .next()
                        .ok_or(CLIErr::OutputDir("Missing value for --output-dir".into()))?;

                    output_dir = Some(PathBuf::from(value));
                }

                "--env-path" | "-env" => {
                    let value = args
                        .next()
                        .ok_or(CLIErr::EnvPath("Missing value for --env-path".into()))?;

                    env_path = Some(PathBuf::from(value));
                }

                "--help" | "-h" => {
                    help = true;
                }

                "--simulate" | "-s" => {
                    simulate = true;
                }

                _ => {
                    return Err(CLIErr::Unknown(format!("Unknown argument: {}", arg)));
                }
            }
        }

        load_env(&mut matrix_path, &mut output_dir, env_path)?;

        Ok(CLI {
            alpha: alpha.ok_or(CLIErr::Alpha("Missing --alpha argument".into()))?,
            epsilon: epsilon.ok_or(CLIErr::Epsilon("Missing --epsilon argument".into()))?,
            treshold: treshold.ok_or(CLIErr::Treshold("Missing --treshold argument".into()))?,
            matrix_path: matrix_path
                .ok_or(CLIErr::MatrixPath("Missing --matrix-path argument".into()))?,
            output_dir: output_dir
                .ok_or(CLIErr::OutputDir("Missing --output-dir argument".into()))?,
            help,
            simulate,
        })
    }

    pub fn run() {
        match CLI::try_new() {
            Ok(cli) => {
                if cli.help {
                    println!("{}", HELP);
                    return;
                }

                if !cli.simulate {
                    match parse_file(&cli.matrix_path, market_parser, cli.alpha, cli.treshold) {
                        Ok(matrix) => match matrix.stationary_distribution(cli.epsilon) {
                            Ok((vec, steps)) => {
                                println!("Sum of distribution = {}", vec.iter().sum::<f64>());
                                println!("Step : {}", steps);

                                if let Err(e) = matrix.dump(&cli.output_dir.join("save.mtx")) {
                                    eprintln!("{}", e)
                                }
                            }
                            Err(e) => eprintln!("{}", e),
                        },
                        Err(e) => eprintln!("{}", e),
                    }
                } else {
                    println!("");

                    if let Err(e) = simulation(
                        cli.alpha,
                        cli.epsilon,
                        cli.treshold,
                        &cli.matrix_path,
                        &cli.output_dir,
                    ) {
                        eprintln!("Simulation failed due to : [{}]", e);
                    } else {
                        println!("Successfully run the simulation.");
                        if let Err(e) = generate(
                            &cli.output_dir.join("results.csv"),
                            &cli.output_dir.join("chart.png"),
                        ) {
                            eprintln!("{}", e);
                        };
                    }
                }
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}

/// Load variables from `.env` file at the root of the project.
fn load_env(
    matrix_path: &mut Option<PathBuf>,
    output_dir: &mut Option<PathBuf>,
    env_path: Option<PathBuf>,
) -> Result<(), CLIErr> {
    if let Some(path) = env_path {
        let _ = fs::read_to_string(&path)
            .map(|content| parse_and_set_env(&content))
            .map_err(|e| {
                CLIErr::EnvPath(format!(
                    "Failed to load the env file {} due to {}",
                    path.display(),
                    e
                ))
            })?;

        if matrix_path.is_none() {
            *matrix_path = Some(PathBuf::from(std::env::var("MATRIX_PATH").map_err(
                |e| {
                    CLIErr::MatrixPath(format!(
                        "Failed to get the environment variable `MATRIX_PATH` from the file {} due to {}",
                        path.display(), e
                    ))
                },
            )?))
        }

        if output_dir.is_none() {
            *output_dir = Some(PathBuf::from(std::env::var("OUTPUT_DIR").map_err(
                |e| {
                    CLIErr::OutputDir(format!(
                        "Failed to get the environment variable `OUTPUT_DIR` from the file {} due to {}",
                        path.display(), e
                    ))
                },
            )?))
        }
    }
    Ok(())
}

/// Parse the `.env` file and set them as environnment variables.
fn parse_and_set_env(content: &str) {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(eq_idx) = line.find('=') {
            let key = line[..eq_idx].trim();
            let mut value = line[eq_idx + 1..].trim();

            // Delete simple quotes and double quotes.
            if (value.starts_with('"') && value.ends_with('"') && value.len() >= 2)
                || (value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2)
            {
                value = &value[1..value.len() - 1];
            }

            unsafe {
                std::env::set_var(key, value);
            }
        }
        // Lines without '=' are skiped
    }
}
