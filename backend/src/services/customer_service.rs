// 客户主数据业务逻辑：客户编码自动生成（CUS-UUID短尾）、客户搜索、活跃客户筛选
// 客户编码支持手动指定与自动生成两种模式，手动指定时需校验唯一性

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::dto::common::PaginationParams;
use crate::dto::customer_dto::{
    CreateCustomerRequest, CustomerFilterParams, UpdateCustomerRequest,
};
use crate::error::AppError;
use crate::models::customer::Customer;
use crate::repositories::customer_repo::CustomerRepo;

pub struct CustomerService;

impl CustomerService {
    fn generate_code() -> String {
        let serial = Uuid::new_v4().to_string();
        format!("CUS-{}", &serial[..8])
    }

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

    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Customer, AppError> {
        CustomerRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::CustomerNotFound(format!("Customer id={} not found", id)))
    }

    pub async fn list(
        pool: &SqlitePool,
        filter: &CustomerFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<Customer>, u64), AppError> {
        CustomerRepo::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

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

    pub async fn list_active(pool: &SqlitePool) -> Result<Vec<Customer>, AppError> {
        CustomerRepo::find_all_active(pool)
            .await
            .map_err(AppError::from)
    }
}
