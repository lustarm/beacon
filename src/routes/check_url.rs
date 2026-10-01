use axum::{Json, response::IntoResponse};
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
            match serde_json::to_value(GetStatusResult {
                url,
                online: true,
            }) {
                Ok(value) => Json(value),
                Err(_) => return (
                    StatusCode::OK,
                    Json(serde_json::json!({ "error": true, "message": "invalid JSON" })),
                )
            }
        ),
        Err(err) => {
            if err.is_dns() {
                error!(logger, "Failed to lookup DNS information for address {}", url);
                return (
                    StatusCode::OK,
                    Json(serde_json::json!({ "error": true, "message": "failed to lookup DNS information" })),
                )
            }

            error!(logger, "Failed to send HTTP request to url {}", url);
            (
                StatusCode::OK,
                Json(serde_json::json!({ "error": false, "message": "failed to send http request" })),
            )
        }
    }
}
