//! Portal DTOs.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreatePortalAccountRequest {
    pub party_type: String,   // supplier | customer
    pub party_id: i64,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct PortalLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AcceptPurchaseRequest {
    pub notes: Option<String>,
}
