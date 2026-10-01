use axum::{Json, response::{Response, IntoResponse}};
use reqwest::{Client, StatusCode};
use slog::{error, info, o, Drain};

use crate::routes::{GetStatusResult, GetStatusRequest};


pub async fn check_url(Json(payload): Json<GetStatusRequest>) -> impl IntoResponse {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let logger = slog::Logger::root(drain, o!());

    if payload.url.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": true })),
        );
    }

    let url = payload.url;

    info!(logger, "Requesting status code from {}", url);
    let client = Client::builder()
        .build()
        .unwrap();

    match client.get(&url).send().await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::to_value(GetStatusResult {
                url,
                online: true,
            }).unwrap()),
        ),
        Err(err) => {
            if err.is_dns() {
                error!(logger, "Failed to lookup DNS information for address {}", url);
            } else {
                error!(logger, "Failed to send HTTP request to url {}", url);
            }
            (
                StatusCode::OK,
                Json(serde_json::json!({ "online": false, "message": "website may not be online" })),
            )
        }
    }
}
