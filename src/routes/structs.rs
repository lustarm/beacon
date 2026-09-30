use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetStatusRequest {
    pub url: String
}

#[derive(Serialize, Deserialize)]
pub struct GetStatusResult {
    pub url: String,
    pub online: bool
}




