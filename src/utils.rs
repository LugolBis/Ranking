use std::fs;

// Absolute path to the .env file (demined at compile time)
const ENV_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.env");

/// Loas variables from `.env` file at the root of the project.
pub fn load_env() -> bool {
    fs::read_to_string(ENV_PATH)
        .map(|content| parse_and_set_env(&content))
        .is_ok()
}

// Parse the `.env` file and set them as environnment variables.
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
