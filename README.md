# TrueWallet Voucher Library for Rust

Designed to be efficient but featureful.

## Example

```rust
use tw_voucher_rs::VoucherClient;

// In case of static mobile number
let voucher_client = VoucherClient::new("0661115555", None);
let response = voucher_client.redeem("[LINK TO VOUCHER / VOUCHER ID]").await.unwrap();

// You can set new mobile number to achieve dynamic number but do note that `voucher_client` need to be mutable thus not recommended
voucher_client.mobile = "0112223333".to_string();

// In case of dynamic mobile number. Please note that below code create NEW HTTP CLIENT everytime it is call.
let response = VoucherClient::one_shot_redeem("0661115555", "[LINK TO VOUCHER / VOUCHER ID]").await.unwrap();
// If you want to avoid that, do:
let shared_client = VoucherClient::new_http_client_builder(tw_voucher_rs::DEFAULT_USER_AGENT).build();
let response = VoucherClient::one_shot_redeem_with_client(shared_client, "0661115555", "[LINK TO VOUCHER / VOUCHER ID]").await.unwrap();

println!("{}", response.data.unwrap().my_ticket.unwrap().amount_bath);
```
