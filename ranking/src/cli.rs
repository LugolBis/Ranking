use crate::errors::CLIErr;
use crate::parser::{api::parse_file, market::market_parser};
use crate::simulation::{Alpha, Threshold, simulation};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CLI {
    #[serde(default)]
    alpha: Alpha,
    #[serde(default)]
    treshold: Threshold,
    epsilon: f64,
    matrix_path: PathBuf,
    output_dir: PathBuf,
    #[serde(default)]
    help: bool,
    #[serde(default)]
    simulate: bool,
    #[serde(default)]
    load: bool,
    #[serde(default)]
    group_count: u64,
}

const HELP: &str = r#"
    Usage : ranking <ARGS>
    Example : ranking --config /path/to/config.json

    ARGS :
        --config, -config, --c, -c : The path to the JSON configuration file.
        --help, -help, --h, -h : Display this message.
        --simulate, -simulate, --s, -s : Simulate the evolution of the stationary distribution convergence according to alpha and treshold.
        --load, -load, --l, -l : Run the simulation by loading the previous matrix with edges removed.

    Config file example :
    {
        "alpha": {
            "start": 0.0,
            "end": 0.90,
            "step": 0.01
        },
        "treshold": {
            "start": 0.0,
            "end": 0.20,
            "step": 0.001
        },
        "epsilon": 1e-6,
        "matrix_path": "/path/to/your/matrix.mtx",
        "output_dir": "/etc/path/to/your/simulation_output_dir/"
    }
"#;

impl Default for CLI {
    // A default implementation used for display the help message.
    fn default() -> Self {
        CLI {
            alpha: Alpha::default(),
            treshold: Threshold::default(),
            epsilon: 0f64,
            matrix_path: PathBuf::new(),
            output_dir: PathBuf::new(),
            help: true,
            simulate: false,
            load: false,
            group_count: 1,
        }
    }
}

impl CLI {
    /// Parse the CLI arguments.
    fn try_new() -> Result<CLI, CLIErr> {
        let mut conf_path: Option<PathBuf> = None;
        let mut help = false;
        let mut simulate = false;
        let mut load = false;

        let mut args = std::env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--config" | "-config" | "--c" | "-c" => {
                    conf_path = Some(PathBuf::from(
                        args.next()
                            .ok_or(CLIErr::Config("Missing value for --matrix-path".into()))?,
                    ));
                }

                "--help" | "-help" | "--h" | "-h" => {
                    help = true;
                }

                "--simulate" | "-simulate" | "--s" | "-s" => {
                    simulate = true;
                }

                "--load" | "-load" | "--l" | "-l" => {
                    load = true;
                }

                _ => {
                    return Err(CLIErr::Unknown(format!("Unknown argument: {}", arg)));
                }
            }
        }

        if help {
            return Ok(CLI::default());
        }

        let config_path = conf_path.ok_or(CLIErr::Config("Missing --config argument".into()))?;

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| CLIErr::Config(format!("Cannot read config file : {}", e)))?;

        let mut cli: CLI = serde_json::from_str(&content)
            .map_err(|e| CLIErr::Config(format!("Invalid JSON : {}", e)))?;
        cli.simulate = simulate;
        cli.load = load;

        if cli.alpha.start() > cli.alpha.end() {
            return Err(CLIErr::Alpha("alpha.start must be <= alpha.end".into()));
        }
        if cli.treshold.start() > cli.treshold.end() {
            return Err(CLIErr::Treshold("threshold must be in [0, 100]".into()));
        }
        if cli.alpha.step() == 0f64 {
            return Err(CLIErr::Alpha("alpha.step must be greater than 0".into()));
        }
        if cli.treshold.step() == 0f64 {
            return Err(CLIErr::Alpha("treshold.step must be greater than 0".into()));
        }
        if cli.epsilon == 0.0 {
            return Err(CLIErr::Alpha("epsilon must be greater than 0.0".into()));
        }

        Ok(cli)
    }

    pub fn run() {
        match CLI::try_new() {
            Ok(cli) => {
                if cli.help {
                    println!("{}", HELP);
                    return;
                }

                if !cli.simulate {
                    match parse_file(&cli.matrix_path, market_parser, cli.alpha.end()) {
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
                    println!();

                    if let Err(e) = simulation(
                        cli.alpha,
                        cli.treshold,
                        cli.epsilon,
                        &cli.matrix_path,
                        &cli.output_dir,
                        cli.load,
                    ) {
                        eprintln!("Simulation failed due to : [{}]", e);
                    } else {
                        println!("Successfully run the simulation !");
                        println!(
                            "The results of the simulation are written in '{:#?}'",
                            &cli.output_dir.join("results.csv")
                        );
                    }
                }
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}
