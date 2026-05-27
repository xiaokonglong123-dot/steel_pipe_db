use sqlx::SqlitePool;
use uuid::Uuid;

use crate::dto::common::PaginationParams;
use crate::dto::customer_dto::{
    CreateCustomerRequest, CustomerFilterParams, UpdateCustomerRequest,
};
use crate::error::AppError;
use crate::models::customer::Customer;
use crate::repositories::customer_repo::CustomerRepo;

/// Customer management service — handles CRUD, search, and query ops for customers.
/// Auto-generates customer codes (`CUS-` prefix + UUID short code) or validates
/// custom codes for uniqueness on creation.
pub struct CustomerService;

impl CustomerService {
    fn generate_code() -> String {
        let serial = Uuid::new_v4().to_string();
        format!("CUS-{}", &serial[..8])
    }

    /// Creates a customer. If `customer_code` is provided, validates uniqueness;
    /// otherwise auto-generates one.
    ///
    /// # Errors
    /// - `AppError::CustomerCodeDuplicate` — customer code already exists
    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateCustomerRequest,
    ) -> Result<Customer, AppError> {
        let code = match &dto.customer_code {
            Some(c) if !c.is_empty() => {
                if CustomerRepo::find_by_code(pool, c)
                    .await
                    .map_err(AppError::from)?
                    .is_some()
                {
                    return Err(AppError::CustomerCodeDuplicate(format!(
                        "Customer code '{}' already exists",
                        c
                    )));
                }
                c.clone()
            }
            _ => Self::generate_code(),
        };

        CustomerRepo::create(pool, dto, &code)
            .await
            .map_err(AppError::from)
    }

    /// Updates customer info. Won't touch soft-deleted customers.
    ///
    /// # Errors
    /// - `AppError::CustomerNotFound` — ID doesn't exist or was deleted
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateCustomerRequest,
    ) -> Result<Customer, AppError> {
        let existing = CustomerRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::CustomerNotFound(format!("Customer id={} not found", id)))?;

        if existing.deleted_at.is_some() {
            return Err(AppError::CustomerNotFound(format!(
                "Customer id={} has been deleted",
                id
            )));
        }

        CustomerRepo::update(pool, id, dto)
            .await
            .map_err(AppError::from)
    }

    /// Soft-deletes a customer. Can't double-delete.
    ///
    /// # Errors
    /// - `AppError::CustomerNotFound` — ID doesn't exist or was already deleted
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        let existing = CustomerRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::CustomerNotFound(format!("Customer id={} not found", id)))?;

        if existing.deleted_at.is_some() {
            return Err(AppError::CustomerNotFound(format!(
                "Customer id={} has been deleted",
                id
            )));
        }

        CustomerRepo::delete(pool, id)
            .await
            .map_err(AppError::from)
    }

    /// Fetches a customer by ID.
    ///
    /// # Errors
    /// - `AppError::CustomerNotFound` — ID doesn't exist or was deleted
    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Customer, AppError> {
        CustomerRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::CustomerNotFound(format!("Customer id={} not found", id)))
    }

    /// Paginates customers with filters for code, name, contact, etc.
    pub async fn list(
        pool: &SqlitePool,
        filter: &CustomerFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<Customer>, u64), AppError> {
        CustomerRepo::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

    /// Searches customers by keyword — name, code, contact, etc. — fuzzy match.
    ///
    /// # Errors
    /// - `AppError::Validation` — search query is empty
    pub async fn search(
        pool: &SqlitePool,
        query: &str,
    ) -> Result<Vec<Customer>, AppError> {
        if query.trim().is_empty() {
            return Err(AppError::Validation("Search query is required".into()));
        }
        CustomerRepo::search(pool, query)
            .await
            .map_err(AppError::from)
    }

    /// Lists all active (non-deleted) customers with basic info for dropdown selects.
    pub async fn list_active(pool: &SqlitePool) -> Result<Vec<Customer>, AppError> {
        CustomerRepo::find_all_active(pool)
            .await
            .map_err(AppError::from)
    }
}
