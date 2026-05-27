use serde::Deserialize;
use validator::Validate;

/// Create customer request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCustomerRequest {
    /// Customer code (auto-generated if empty).
    pub customer_code: Option<String>,
    /// Customer name.
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

/// Update customer info request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCustomerRequest {
    /// Customer name.
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

/// Customer list filter params.
#[derive(Debug, Deserialize)]
pub struct CustomerFilterParams {
    /// Full-text search.
    pub q: Option<String>,
    /// Filter by active status.
    pub is_active: Option<bool>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}
