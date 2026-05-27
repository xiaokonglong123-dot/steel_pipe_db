use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Customer DB row. The folks buying pipes from us.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Customer {
    pub id: i64,
    /// Customer code — unique across the system.
    pub customer_code: String,
    /// Customer name.
    pub name: String,
    /// Contact person.
    pub contact_person: Option<String>,
    /// Contact phone number.
    pub phone: Option<String>,
    /// Email address.
    pub email: Option<String>,
    /// Physical address.
    pub address: Option<String>,
    /// Whether this customer is active.
    pub is_active: bool,
    /// Free-form notes.
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}
