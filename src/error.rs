use thiserror::Error;

use crate::response::APIResponse;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Most likely: Cloudflare Forbidden")]
    Forbidden,

    #[error("reqwest throw an error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("serde_json throw an error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Failed to deserialize: {0}\nContent: {1}")]
    Deserialize(serde_json::Error, String),

    #[error("HTTP Response Status Code is not 200: {0}")]
    StatusCode(u16, String),

    #[error("Voucher Error: {0}")]
    Voucher(Box<APIResponse>),
}
