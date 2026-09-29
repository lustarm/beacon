use axum::Json;
use reqwest::Client;
use slog::{Logger, error, info};
use tokio::time::Duration;

use crate::routes::structs::GetStatusRequest;

pub async fn check_url(Json(payload): Json<GetStatusRequest>, timeout: u32, logger: Logger) -> &'static str {
    let url = payload.url;

    info!(logger, "Requesting status code from {}", url);
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout.into()))
        .build()
        .unwrap();

    match client.get(&url).send().await {
        Ok(_) => return "website is online",
        Err(err) => {
            if err.is_dns() {
                error!(logger, "Failed to lookup DNS information for address {}", url);
            } else {
                error!(logger, "Failed to send HTTP request to url {}", url);
            }
            return "website may not be online"
        }
    }
}
