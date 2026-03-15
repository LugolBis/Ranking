use crate::{
    errors::{CLIErr, CSCErr},
    matrix::CSC,
};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::{fs, path::PathBuf};

/// Absolute path to the .env file (demined at compile time)
const ENV_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.env");

/// Parse the CLI arguments.
pub fn parse_args() -> Result<(f64, f64, f64, Option<PathBuf>), CLIErr> {
    let mut args = std::env::args().skip(1);
    let alpha = args
        .next()
        .ok_or(CLIErr::Alpha("Missing first argument : alpha".into()))?
        .parse::<f64>()
        .map_err(|e| CLIErr::Alpha(format!("Failed to parse alpha : {}", e)))?;
    let epsilon = args
        .next()
        .ok_or(CLIErr::Epsilon("Missing second argument : epsilon".into()))?
        .parse::<f64>()
        .map_err(|e| CLIErr::Epsilon(format!("Failed to parse epsilon : {}", e)))?;
    let treshold = args
        .next()
        .ok_or(CLIErr::Treshold("Missing third argument : treshold".into()))?
        .parse::<f64>()
        .map_err(|e| CLIErr::Treshold(format!("Failed to parse treshold : {}", e)))?;

    Ok((
        alpha,
        epsilon,
        treshold,
        args.next().and_then(|val| Some(PathBuf::from(val))),
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

/// Write the CSC matrix as the Matrix Format in the given `output_path` file.
pub fn dump_matrix(matrix: CSC, output_path: PathBuf) -> Result<(), CSCErr> {
    let file = File::create(&output_path)
        .map_err(|e| CSCErr::Dump(format!("{} with path {}", e, output_path.display())))?;

    let mut buffer = BufWriter::new(file);
    buffer
        .write_all("%%MatrixMarket matrix coordinate pattern general\n".as_bytes())
        .map_err(|e| CSCErr::Dump(format!("Failed to write headers due to : {}", e)))?;

    let shape = matrix.get_shape();
    writeln!(
        buffer,
        "{} {} {}",
        shape.rows(),
        shape.columns(),
        matrix.get_count()
    )
    .map_err(|e| CSCErr::Dump(format!("Failed to write shape due to : {}", e)))?;

    let mut iterator = matrix.get_columns().iter().enumerate();

    while let Some((col_idx, opt_col)) = iterator.next() {
        if let Some(column) = opt_col {
            for value in column.rows.iter() {
                writeln!(buffer, "{} {}", value.get_row_index() + 1, col_idx + 1).map_err(|e| {
                    CSCErr::Dump(format!(
                        "Failed to write the value {} at row={} column={} due to {}",
                        value.get_value(),
                        value.get_row_index(),
                        col_idx,
                        e
                    ))
                })?;
            }
        }
    }

    Ok(())
}
