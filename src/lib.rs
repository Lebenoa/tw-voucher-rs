mod error;
mod response;

use reqwest::{Client, ClientBuilder, StatusCode};

use crate::{error::Error, response::APIResponse};

const DEFAULT_USER_AGENT: &str = "Bun/1.2.21";

pub struct VoucherClient {
    http_client: Client,
    mobile: String,
}

impl VoucherClient {
    /// Create new Voucher client
    ///
    /// # Example
    ///
    /// ```rs
    /// VoucherClient::new("0669991111", Some("custom user agent"));
    /// ```
    pub fn new<S: Into<String>>(mobile: S, user_agent: Option<S>) -> Result<Self, Error> {
        let ua = if let Some(u) = user_agent {
            u.into()
        } else {
            DEFAULT_USER_AGENT.to_string()
        };

        #[cfg(debug_assertions)]
        let http_client = ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .user_agent(ua)
            .use_rustls_tls()
            .build()?;

        #[cfg(not(debug_assertions))]
        let http_client = ClientBuilder::new()
            .user_agent(ua)
            .use_rustls_tls()
            .build()?;

        Ok(VoucherClient {
            http_client,
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
    /// VoucherClient::new(http_client.clone(), "0669991111")
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
        let body = self.create_redeem_request_body(extracted_id);
        let response = self
            .http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

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
                _ if !api_response.status.code().is_success() => {
                    return Err(Error::Voucher(api_response.status.message));
                }
                _ => {
                    return Err(Error::StatusCode(status.as_u16(), body));
                }
            }
        }

        Ok(api_response)
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
                _ if !api_response.status.code().is_success() => {
                    return Err(Error::Voucher(api_response.status.message));
                }
                _ => {
                    return Err(Error::StatusCode(status.as_u16(), body));
                }
            }
        }

        Ok(api_response)
    }

    pub async fn one_shot_redeem(mobile: &str, voucher: &str) -> Result<APIResponse, Error> {
        let vc = VoucherClient::new(mobile, None)?;
        vc.redeem(voucher).await
    }

    pub async fn one_shot_redeem_with_client(
        client: Client,
        mobile: &str,
        voucher: &str,
    ) -> Result<APIResponse, Error> {
        let vc = VoucherClient::new_with_client(client, mobile);
        vc.redeem(voucher).await
    }

    pub async fn one_shot_verify(mobile: &str, voucher: &str) -> Result<APIResponse, Error> {
        let vc = VoucherClient::new(mobile, None)?;
        vc.verify(voucher).await
    }

    pub async fn one_shot_verify_with_client(
        client: Client,
        mobile: &str,
        voucher: &str,
    ) -> Result<APIResponse, Error> {
        let vc = VoucherClient::new_with_client(client, mobile);
        vc.verify(voucher).await
    }

    #[inline]
    fn extract_id(link: &str) -> &str {
        match link.split_once("?v=") {
            Some((_prefix, id)) => id,
            None => link,
        }
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
