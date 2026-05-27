use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Supplier DB row. The folks selling pipes to us.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Supplier {
    pub id: i64,
    /// Supplier code — unique across the system.
    pub supplier_code: String,
    /// Supplier name.
    pub name: String,
    /// Contact person.
    pub contact_person: Option<String>,
    /// Contact phone number.
    pub phone: Option<String>,
    /// Email address.
    pub email: Option<String>,
    /// Physical address.
    pub address: Option<String>,
    /// Whether this supplier is active.
    pub is_active: bool,
    /// Free-form notes.
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}
