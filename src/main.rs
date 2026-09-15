use std::thread::sleep;

use reqwest::{Result, StatusCode};

pub struct Target {
    name: String,
    url: String,
    custom_interval_seconds: Option<u32>,
    custom_timout_seconds: Option<u32>
}

pub struct Defaults {
    interval_seconds: u32,
    timout_seconds: u32
}

pub struct Config {
    defaults: Defaults,
    targets: Vec<Target>
}

fn read_config() -> Config {
    let mut targs: Vec<Target> = Vec::new();

    targs.push(Target{
        name: "Google".to_string(),
        url: "https://google.com/".to_string(),
        custom_interval_seconds: None,
        custom_timout_seconds: None
    });

    // have to add "hacker" buzz words
    println!("[config] Loaded config correctly");

    Config {
        defaults: Defaults {
            interval_seconds: 30,
            timout_seconds: 5
        },
        targets: targs
    }
}

async fn get_status(url: &str) -> StatusCode {
    println!("[info] Requesting status code from {}", url);
    reqwest::get(url)
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

            match target.custom_interval_seconds {
                Some(x) => sleep_time = x,
                None => sleep_time = config.defaults.interval_seconds
            }

            match get_status(url).await {
                StatusCode::OK => println!("[status] Status code Ok from {}:{}",
                    name, url),
                _ => println!("[error] Failed to get status code from {}:{}",
                    name, url),
            }
            sleep(std::time::Duration::from_secs(sleep_time.into()));
        }
    }
}
