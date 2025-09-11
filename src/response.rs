#![allow(dead_code)]
use std::fmt::Display;

use serde::Deserialize;

#[derive(PartialEq)]
pub enum StatusCode<'a> {
    Success,

    /// This voucher is already out of stock.
    VoucherOutOfStock,

    /// This voucher is already expired
    VoucherExpired,

    /// You cannot redeem you own voucher.
    CannotGetOwnVoucher,

    /// Provided voucher does not exists
    VoucherNotFound,

    /// Other StatusCode that aren't list in the library.
    Other(&'a str),
}

impl<'a> StatusCode<'a> {
    pub fn is_success(&self) -> bool {
        matches!(self, StatusCode::Success)
    }
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Deserialize, Debug)]
pub struct Status {
    pub message: String,
    code: String,
}

impl Status {
    pub fn code(&self) -> StatusCode {
        match self.code.as_str() {
            "VOUCHER_EXPIRED" => StatusCode::VoucherExpired,
            "VOUCHER_OUT_OF_STOCK" => StatusCode::VoucherOutOfStock,
            "CANNOT_GET_OWN_VOUCHER" => StatusCode::CannotGetOwnVoucher,
            "VOUCHER_NOT_FOUND" => StatusCode::VoucherNotFound,
            "SUCCESS" => StatusCode::Success,
            _ => StatusCode::Other(&self.code),
        }
    }
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Deserialize, Debug)]
pub struct Voucher {
    pub voucher_id: String,
    pub amount_baht: String,
    pub redeemed_amount_baht: String,
    pub member: u16,
    pub status: String,
    pub link: String,
    pub detail: String,
    pub expire_date: u64,
    pub r#type: String,
    pub redeemed: u16,
    pub available: u16,
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Deserialize, Debug)]
pub struct OwnerProfile {
    pub full_name: String,
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Deserialize, Debug)]
pub struct RedeemerProfile {
    pub mobile_number: String,
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Deserialize, Debug)]
pub struct Ticket {
    pub mobile: String,
    pub update_date: u64,
    pub amount_baht: String,
    pub full_name: String,
    pub profile_pic: Option<String>,
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Deserialize, Debug)]
pub struct Data {
    pub voucher: Voucher,
    pub owner_profile: OwnerProfile,
    pub redeemer_profile: Option<RedeemerProfile>,
    pub my_ticket: Option<Ticket>,
    pub tickets: Vec<Ticket>,
}

impl Data {
    pub fn is_my_ticket(&self) -> bool {
        self.my_ticket.is_some()
    }
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Deserialize, Debug)]
pub struct APIResponse {
    pub status: Status,
    pub data: Option<Data>,
}

impl APIResponse {
    pub fn is_valid_from_verify(&self) -> bool {
        if !self.status.code().is_success() {
            return false;
        }

        let Some(data) = self.data.as_ref() else {
            return false;
        };

        data.voucher.status == "active" && data.voucher.available > 0
    }
}

impl Display for APIResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.status.message)
    }
}

#[cfg(feature = "serialize")]
pub fn serialize(resp: &APIResponse) -> Result<String, serde_json::Error> {
    serde_json::to_string(resp)
}

#[cfg(feature = "serialize")]
pub fn serialize_pretty(resp: &APIResponse) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(resp)
}
