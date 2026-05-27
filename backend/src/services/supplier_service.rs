use sqlx::SqlitePool;
use uuid::Uuid;

use crate::dto::common::PaginationParams;
use crate::dto::supplier_dto::{
    CreateSupplierRequest, SupplierFilterParams, UpdateSupplierRequest,
};
use crate::error::AppError;
use crate::models::supplier::Supplier;
use crate::repositories::supplier_repo::SupplierRepo;

/// Supplier management service — handles CRUD, search, and query ops for suppliers.
/// Auto-generates supplier codes (`SUP-` prefix + UUID short code) or validates
/// custom codes for uniqueness on creation.
pub struct SupplierService;

impl SupplierService {
    fn generate_code() -> String {
        let serial = Uuid::new_v4().to_string();
        format!("SUP-{}", &serial[..8])
    }

    /// Creates a supplier. If `supplier_code` is provided, validates uniqueness;
    /// otherwise auto-generates one.
    ///
    /// # Errors
    /// - `AppError::SupplierCodeDuplicate` — supplier code already exists
    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateSupplierRequest,
    ) -> Result<Supplier, AppError> {
        let code = match &dto.supplier_code {
            Some(c) if !c.is_empty() => {
                if SupplierRepo::find_by_code(pool, c)
                    .await
                    .map_err(AppError::from)?
                    .is_some()
                {
                    return Err(AppError::SupplierCodeDuplicate(format!(
                        "Supplier code '{}' already exists",
                        c
                    )));
                }
                c.clone()
            }
            _ => Self::generate_code(),
        };

        SupplierRepo::create(pool, dto, &code)
            .await
            .map_err(AppError::from)
    }

    /// Updates supplier info. Won't touch soft-deleted suppliers.
    ///
    /// # Errors
    /// - `AppError::SupplierNotFound` — ID doesn't exist or was deleted
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateSupplierRequest,
    ) -> Result<Supplier, AppError> {
        let existing = SupplierRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::SupplierNotFound(format!("Supplier id={} not found", id)))?;

        if existing.deleted_at.is_some() {
            return Err(AppError::SupplierNotFound(format!(
                "Supplier id={} has been deleted",
                id
            )));
        }

        SupplierRepo::update(pool, id, dto)
            .await
            .map_err(AppError::from)
    }

    /// Soft-deletes a supplier. Can't double-delete.
    ///
    /// # Errors
    /// - `AppError::SupplierNotFound` — ID doesn't exist or was already deleted
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        let existing = SupplierRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::SupplierNotFound(format!("Supplier id={} not found", id)))?;

        if existing.deleted_at.is_some() {
            return Err(AppError::SupplierNotFound(format!(
                "Supplier id={} has been deleted",
                id
            )));
        }

        SupplierRepo::delete(pool, id)
            .await
            .map_err(AppError::from)
    }

    /// Fetches a supplier by ID.
    ///
    /// # Errors
    /// - `AppError::SupplierNotFound` — ID doesn't exist or was deleted
    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Supplier, AppError> {
        SupplierRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::SupplierNotFound(format!("Supplier id={} not found", id)))
    }

    /// Paginates suppliers with filters for code, name, contact, etc.
    pub async fn list(
        pool: &SqlitePool,
        filter: &SupplierFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<Supplier>, u64), AppError> {
        SupplierRepo::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

    /// Searches suppliers by keyword — name, code, contact, etc. — fuzzy match.
    ///
    /// # Errors
    /// - `AppError::Validation` — search query is empty
    pub async fn search(
        pool: &SqlitePool,
        query: &str,
    ) -> Result<Vec<Supplier>, AppError> {
        if query.trim().is_empty() {
            return Err(AppError::Validation("Search query is required".into()));
        }
        SupplierRepo::search(pool, query)
            .await
            .map_err(AppError::from)
    }

    /// Lists all active (non-deleted) suppliers for dropdown selects.
    pub async fn list_active(pool: &SqlitePool) -> Result<Vec<Supplier>, AppError> {
        SupplierRepo::find_all_active(pool)
            .await
            .map_err(AppError::from)
    }
}
