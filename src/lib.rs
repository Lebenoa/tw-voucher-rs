pub mod error;
pub mod response;

use std::collections::HashMap;

use reqwest::{Client, ClientBuilder, StatusCode, Url};

use crate::{error::Error, response::APIResponse};

pub const DEFAULT_USER_AGENT: &str = "Bun/1.2.21";

pub struct VoucherClient {
    pub http_client: Client,
    pub mobile: String,
}

impl VoucherClient {
    /// Construct usable, default `reqwest::ClientBuilder`. Perfect for sharing across all `VoucherClient`
    pub fn new_http_client_builder(user_agent: &str) -> ClientBuilder {
        ClientBuilder::new().user_agent(user_agent).use_rustls_tls()
    }

    pub fn get_http_client(&self) -> Client {
        self.http_client.clone()
    }

    /// Create new voucher client with new `reqwest::Client`  
    /// For multiple voucher client, consider using `VoucherClient::new_http_client_builder` and calling `VoucherClient::new_with_client` instead
    ///
    /// # Example
    ///
    /// ```rs
    /// VoucherClient::new("0669991111", Some("custom user agent"));
    /// ```
    pub fn new<S: Into<String>>(mobile: S, user_agent: Option<&str>) -> Result<Self, Error> {
        let mut http_client_builder = if let Some(u) = user_agent {
            Self::new_http_client_builder(u)
        } else {
            Self::new_http_client_builder(DEFAULT_USER_AGENT)
        };

        #[cfg(debug_assertions)]
        {
            http_client_builder = http_client_builder
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
        }

        Ok(VoucherClient {
            http_client: http_client_builder.build()?,
            mobile: mobile.into(),
        })
    }

    /// Use provided `reqwest::Client` instead of creating a new one
    ///
    /// # Important
    ///
    /// Use `rustls-tls` otherwise cloudflare will block the request
    ///
    /// # Example
    ///
    /// ```rs
    /// let http_client = reqwest::ClientBuilder::new()
    ///    .user_agent("MyBin/1.0")
    ///    .use_rustls_tls()
    ///    .build()
    ///    .unwrap();
    ///
    /// // or
    ///
    /// let http_client = VoucherClient::new_http_client_builder().build().unwrap();
    ///
    /// VoucherClient::new_with_client(http_client.clone(), "0669991111")
    /// ```
    pub fn new_with_client<S: Into<String>>(client: Client, mobile: S) -> Self {
        VoucherClient {
            http_client: client,
            mobile: mobile.into(),
        }
    }

    pub async fn redeem(&self, voucher: &str) -> Result<APIResponse, Error> {
        let extracted_id = Self::extract_id(voucher);
        let url = format!("https://gift.truemoney.com/campaign/vouchers/{extracted_id}/redeem");
        let body = self.create_redeem_request_body(&extracted_id);
        let response = self
            .http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        Self::handle_response(response).await
    }

    pub async fn verify(&self, voucher: &str) -> Result<APIResponse, Error> {
        let extracted_id = Self::extract_id(voucher);
        let url = format!(
            "https://gift.truemoney.com/campaign/vouchers/{extracted_id}/verify?mobile={}",
            self.mobile
        );

        let response = self
            .http_client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        Self::handle_response(response).await
    }

    pub async fn one_shot_redeem(mobile: &str, voucher: &str) -> Result<APIResponse, Error> {
        let extracted_id = Self::extract_id(voucher);
        let url = format!("https://gift.truemoney.com/campaign/vouchers/{extracted_id}/redeem");
        let body = format!(r#"{{"mobile":"{mobile}","voucher_hash":"{extracted_id}"}}"#,);
        let client = Self::new_http_client_builder(DEFAULT_USER_AGENT).build()?;

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        Self::handle_response(response).await
    }

    pub async fn one_shot_redeem_with_client(
        client: Client,
        mobile: &str,
        voucher: &str,
    ) -> Result<APIResponse, Error> {
        let extracted_id = Self::extract_id(voucher);
        let url = format!("https://gift.truemoney.com/campaign/vouchers/{extracted_id}/redeem");
        let body = format!(r#"{{"mobile":"{mobile}","voucher_hash":"{extracted_id}"}}"#,);

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        Self::handle_response(response).await
    }

    pub async fn one_shot_verify(mobile: &str, voucher: &str) -> Result<APIResponse, Error> {
        let extracted_id = Self::extract_id(voucher);
        let url = format!(
            "https://gift.truemoney.com/campaign/vouchers/{extracted_id}/verify?mobile={mobile}",
        );
        let client = Self::new_http_client_builder(DEFAULT_USER_AGENT).build()?;

        let response = client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        Self::handle_response(response).await
    }

    pub async fn one_shot_verify_with_client(
        client: Client,
        mobile: &str,
        voucher: &str,
    ) -> Result<APIResponse, Error> {
        let extracted_id = Self::extract_id(voucher);
        let url = format!(
            "https://gift.truemoney.com/campaign/vouchers/{extracted_id}/verify?mobile={mobile}",
        );

        let response = client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        Self::handle_response(response).await
    }

    #[inline]
    async fn handle_response(response: reqwest::Response) -> Result<APIResponse, Error> {
        let status = response.status();
        if status == StatusCode::FORBIDDEN {
            return Err(Error::Forbidden);
        }

        let body = response.text().await?;
        let api_response: APIResponse = match serde_json::from_str(&body) {
            Ok(a) => a,
            Err(e) => {
                return Err(Error::Deserialize(e, body));
            }
        };

        if status != StatusCode::OK {
            match status {
                _ if !api_response.status.code_as_enum().is_success() => {
                    return Err(Error::Voucher(Box::new(api_response)));
                }
                _ => {
                    return Err(Error::StatusCode(status.as_u16(), body));
                }
            }
        }

        Ok(api_response)
    }

    #[inline]
    fn extract_id(link: &str) -> String {
        if let Ok(url) = Url::parse(link) {
            let mut queries: HashMap<_, _> = url.query_pairs().into_owned().collect();
            if let Some(v) = queries.remove("v") {
                return v;
            }
        }

        link.to_string()
    }

    #[inline]
    fn create_redeem_request_body(&self, voucher_id: &str) -> String {
        format!(
            r#"{{"mobile":"{}","voucher_hash":"{}"}}"#,
            self.mobile, voucher_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn redeem() {
        let voucher_client =
            VoucherClient::new(std::env::var("TEST_NUMBER").unwrap(), None).unwrap();

        let json = voucher_client
            .redeem("https://gift.truemoney.com/campaign/?v=019939cee82f7b6fb3153d8db2b219b98aZ")
            .await
            .unwrap();

        println!("{json:#?}");
    }

    #[tokio::test]
    async fn verify() {
        let voucher_client =
            VoucherClient::new(std::env::var("TEST_NUMBER").unwrap(), None).unwrap();

        let json = voucher_client
            .verify("https://gift.truemoney.com/campaign/?v=0199398724f70dcb1a5563c0b7e27f403o")
            .await
            .unwrap();

        println!("{json:#?}");
    }
}
