// 供应商主数据业务逻辑：供应商编码自动生成（SUP-UUID短尾）、资质状态跟踪、搜索
// 编码规则与客户管理对称，支持手动指定和自动生成两种模式

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::dto::common::PaginationParams;
use crate::dto::supplier_dto::{
    CreateSupplierRequest, SupplierFilterParams, UpdateSupplierRequest,
};
use crate::error::AppError;
use crate::models::supplier::Supplier;
use crate::repositories::supplier_repo::SupplierRepo;

pub struct SupplierService;

impl SupplierService {
    fn generate_code() -> String {
        let serial = Uuid::new_v4().to_string();
        format!("SUP-{}", &serial[..8])
    }

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

    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Supplier, AppError> {
        SupplierRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::SupplierNotFound(format!("Supplier id={} not found", id)))
    }

    pub async fn list(
        pool: &SqlitePool,
        filter: &SupplierFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<Supplier>, u64), AppError> {
        SupplierRepo::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

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

    pub async fn list_active(pool: &SqlitePool) -> Result<Vec<Supplier>, AppError> {
        SupplierRepo::find_all_active(pool)
            .await
            .map_err(AppError::from)
    }
}
