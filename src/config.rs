use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    process::exit,
};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub settings: Settings,
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub editor: String,
    pub enable_quote_grading: bool,
}

/// Loads the config file and returns its contents.
///
/// # Returns
/// A `Config` struct containing the settings of the config file.
pub fn get_config(path: &str) -> Config {
    // Make sure the config file exists
    if !Path::new(path).exists() {
        if let Err(e) = init_config_file(path) {
            eprintln!("Failed to initialize config file: {e}");
            exit(-1);
        }
    }

    // The config file could be not properly initialized :shrug:
    let config_res = load_config(path);

    match config_res {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to parse config file: {e}");
            exit(-1);
        }
    }
}

/// Creates the initial config file and populates it with defaults.
fn init_config_file(path: &str) -> std::io::Result<()> {
    let default = r#"[settings]
# Set this to your favorite editor to write your quotes (vim, nvim, nano, ...)
# You can also set it to "stdin" to read from the standard input,
# or "default" to use the default editor on your machine.
editor = "stdin"

# Set to false to disable the quote grading.
enable_quote_grading = true"#;

    let mut file = File::create(path)?;
    file.write_all(default.as_bytes())?;
    println!("Initialized config file '{}'", path);
    Ok(())
}

/// Loads the config file into a string an parses that string.
///
/// # Returns
/// A `ConfigFile` parsed from the config file.
fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
