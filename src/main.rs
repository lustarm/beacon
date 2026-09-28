use std::{thread::sleep, time::Duration, io::Result};
use reqwest::{StatusCode, Client};
use slog::{Drain, Logger, error, info, o};

mod config;
mod routes;

/* TODO: Backend in rust */
use axum::{
    routing::get,
    Router,
};

async fn get_status(url: &str, timeout: u32, logger: Logger) -> StatusCode {
    info!(logger, "Requesting status code from {}", url);
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout.into()))
        .build()
        .unwrap();

    match client.get(url).send().await {
        Ok(response) => response.status(),
        Err(err) => {
            if err.is_dns() {
                error!(logger, "Failed to lookup DNS information for address {}", url);
            } else {
                error!(logger, "Failed to send HTTP request to url {}", url);
            }
            return StatusCode::GATEWAY_TIMEOUT
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    /* == Logger == */
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let logger = slog::Logger::root(drain, o!());

    /* == Actual logic == */
    let config = config::read_config(logger.clone());

    let t_logger = logger.clone();
    let web_thread = tokio::spawn(async move {
        info!(t_logger, "Starting HTTP server thread");

        let app = Router::new()
            .route("/", get(routes::root));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        info!(t_logger, "Listening for HTTP connections on port 3000");
        let _ = axum::serve(listener, app).await;
    });

    /* == Worker thread == */
    let w_logger = logger.clone();
    let worker_thread = tokio::spawn(async move {
        info!(w_logger, "Starting worker thread");

        loop {
            for target in &config.targets {
                let _name = &target.name;
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

                match get_status(url, timeout_time, logger.clone()).await {
                    StatusCode::OK => {
                        // add to log
                        // have to add logging system now...

                    },
                    _ => ()
                }

                sleep(std::time::Duration::from_secs(sleep_time.into()));
            }
        }
    });

    web_thread.await?;
    worker_thread.await?;

    Ok(())
}
