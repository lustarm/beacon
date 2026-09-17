use std::{fs, thread::sleep, time::Duration};

use reqwest::{Result, StatusCode, Client};
use serde::Deserialize;


/* TODO: Parse TOML */


#[derive(Debug, Deserialize)]
pub struct Target {
    name: String,
    url: String,
    custom_interval_seconds: Option<u32>,
    custom_timeout_seconds: Option<u32>
}

#[derive(Debug, Deserialize)]
pub struct Defaults {
    interval_seconds: u32,
    timeout_seconds: u32
}

#[derive(Debug, Deserialize)]
pub struct Config {
    defaults: Defaults,
    targets: Vec<Target>
}

fn read_config() -> Config {
    let mut targs: Vec<Target> = Vec::new();
    let toml_content = fs::read_to_string("config.toml")
        .expect("[error] No config.toml file");

    let config: Config = toml::from_str(&toml_content)
        .expect("[error] Failed to parse config.toml file");

    if config.targets.len() <= 0 {
        println!("[info] No targets in config file");

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
    println!("[config] Loaded config correctly");

    Config {
        defaults: Defaults {
            interval_seconds: 30,
            timeout_seconds: 5
        },
        targets: targs
    }
}

async fn get_status(url: &str, timeout: u32) -> StatusCode {
    println!("[info] Requesting status code from {}", url);
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout.into()))
        .build()
        .unwrap();

    client.get(url)
        .send()
        .await
        .unwrap()
        .status()
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = read_config();

    loop {
        for target in &config.targets {
            let name = &target.name;
            let url = &target.url;
            let sleep_time;
            let timeout_time;

            match target.custom_interval_seconds {
                Some(x) => sleep_time = x,
                None => sleep_time = config.defaults.interval_seconds
            }

            match target.custom_timeout_seconds {
                Some(x) => timeout_time = x,
                None => timeout_time = config.defaults.timeout_seconds
            }

            match get_status(url, timeout_time).await {
                StatusCode::OK => println!("[status] Status code Ok from {}:{}",
                    name, url),
                _ => println!("[error] Failed to get status code from {}:{}",
                    name, url),
            }
            sleep(std::time::Duration::from_secs(sleep_time.into()));
        }
    }
}
