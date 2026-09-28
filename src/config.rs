use std::fs;

use slog::{Logger, info};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Target {
    pub name: String,
    pub url: String,
    pub custom_interval_seconds: Option<u32>,
    pub custom_timeout_seconds: Option<u32>
}

#[derive(Debug, Deserialize)]
pub struct Defaults {
    pub interval_seconds: u32,
    pub timeout_seconds: u32
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub defaults: Defaults,
    pub targets: Vec<Target>
}

pub fn read_config(logger: Logger) -> Config {
    let mut targs: Vec<Target> = Vec::new();
    let toml_content = fs::read_to_string("config.toml")
        .expect("No config.toml file");

    let config: Config = toml::from_str(&toml_content)
        .expect("Failed to parse config.toml file");

    if config.targets.len() <= 0 {
        info!(logger, "No targets in config file");

        return Config {
            defaults: Defaults {
                interval_seconds: 30,
                timeout_seconds: 5
            },
            targets: targs
        }
    }

    for target in config.targets {
        targs.push(target);
    }

    // have to add "hacker" buzz words
    info!(logger, "Loaded config correctly");

    Config {
        defaults: Defaults {
            interval_seconds: 30,
            timeout_seconds: 5
        },
        targets: targs
    }
}

