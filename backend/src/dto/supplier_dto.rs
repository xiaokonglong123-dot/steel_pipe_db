use serde::Deserialize;
use validator::Validate;

/// Create supplier request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSupplierRequest {
    /// Supplier code (auto-generated if empty).
    pub supplier_code: Option<String>,
    /// Supplier name.
    #[validate(length(min = 1))]
    pub name: String,
    /// Contact person.
    pub contact_person: Option<String>,
    /// Contact phone number.
    pub phone: Option<String>,
    /// Email address.
    #[validate(email)]
    pub email: Option<String>,
    /// Address.
    pub address: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

/// Update supplier info request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSupplierRequest {
    /// Supplier name.
    pub name: Option<String>,
    /// Contact person.
    pub contact_person: Option<String>,
    /// Phone number.
    pub phone: Option<String>,
    /// Email address.
    #[validate(email)]
    pub email: Option<String>,
    /// Address.
    pub address: Option<String>,
    /// Whether active.
    pub is_active: Option<bool>,
    /// Notes.
    pub notes: Option<String>,
}

/// Supplier list filter params.
#[derive(Debug, Deserialize)]
pub struct SupplierFilterParams {
    /// Full-text search.
    pub q: Option<String>,
    /// Filter by active status.
    pub is_active: Option<bool>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}
