use reqwest::{Result, StatusCode};

const TEST_URL: &str = "https://google.com/";

#[tokio::main]
async fn main() -> Result<()> {
    let status_code = reqwest::get(TEST_URL)
        .await?
        .status();
    match status_code {
        StatusCode::OK => println!("[status] Status code Ok from {}", TEST_URL),
        _ => println!("[error] Failed to get status code from {}", TEST_URL),
    }
    Ok(())
}
