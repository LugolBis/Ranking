use std::{fs, path::PathBuf};

use crate::errors::CLIErr;

/// Absolute path to the .env file (demined at compile time)
const ENV_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.env");

/// Parse the CLI arguments.
pub fn parse_args() -> Result<(f64, f64, Option<PathBuf>), CLIErr> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let alpha = args
        .get(0)
        .ok_or(CLIErr::Alpha("Missing first argument : alpha".into()))?
        .parse::<f64>()
        .map_err(|e| CLIErr::Alpha(format!("Failed to parse alpha : {}", e)))?;
    let epsilon = args
        .get(1)
        .ok_or(CLIErr::Epsilon("Missing second argument : epsilon".into()))?
        .parse::<f64>()
        .map_err(|e| CLIErr::Epsilon(format!("Failed to parse epsilon : {}", e)))?;

    Ok((
        alpha,
        epsilon,
        args.get(2).and_then(|val| Some(PathBuf::from(val))),
    ))
}

/// Load variables from `.env` file at the root of the project.
pub fn load_env() -> bool {
    fs::read_to_string(ENV_PATH)
        .map(|content| parse_and_set_env(&content))
        .is_ok()
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
