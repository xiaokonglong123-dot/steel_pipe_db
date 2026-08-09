//! Project services — project charter, WBS, budget transactions.

use sqlx::SqlitePool;

use crate::dto::project_dto::{
    CreateProjectRequest, CreateTransactionRequest, CreateWbsRequest, UpdateWbsProgressRequest,
};
use crate::error::AppError;
use crate::models::project::{Project, ProjectFinancials, ProjectTransaction, WbsElement};
use crate::project::repos::{ProjectRepo, ProjectTxRepo, WbsRepo};

pub struct ProjectService;

impl ProjectService {
    pub async fn create_project(pool: &SqlitePool, tenant_id: i64, dto: &CreateProjectRequest) -> Result<Project, AppError> {
        if dto.name.trim().is_empty() {
            return Err(AppError::Validation("Project name is required".into()));
        }
        let project_no = format!("PRJ-{}-{}", chrono::Utc::now().format("%Y%m%d"), seq(pool, "projects").await?);
        ProjectRepo::create(
            pool, tenant_id, &project_no, dto.name.trim(), dto.description.as_deref(),
            dto.start_date, dto.end_date, dto.manager_id, dto.budget.unwrap_or(0.0),
        )
        .await
        .map_err(AppError::from)
    }

    pub async fn list_projects(pool: &SqlitePool, tenant_id: i64, status: Option<&str>) -> Result<Vec<Project>, AppError> {
        ProjectRepo::list(pool, tenant_id, status).await.map_err(AppError::from)
    }

    pub async fn get_project(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<Project, AppError> {
        ProjectRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Project not found: {}", id)))
    }

    pub async fn update_project_status(pool: &SqlitePool, tenant_id: i64, id: i64, status: &str) -> Result<Project, AppError> {
        if !matches!(status, "planning" | "active" | "on_hold" | "completed" | "cancelled") {
            return Err(AppError::Validation(format!("Invalid project status: {}", status)));
        }
        ProjectRepo::update_status(pool, tenant_id, id, status)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Project not found: {}", id)))
    }

    pub async fn create_wbs(pool: &SqlitePool, tenant_id: i64, project_id: i64, dto: &CreateWbsRequest) -> Result<WbsElement, AppError> {
        // Project must exist.
        Self::get_project(pool, tenant_id, project_id).await?;
        if dto.code.trim().is_empty() || dto.name.trim().is_empty() {
            return Err(AppError::Validation("WBS code and name are required".into()));
        }
        WbsRepo::create(
            pool, tenant_id, project_id, dto.parent_id, dto.code.trim(), dto.name.trim(),
            dto.weight_pct, dto.start_date, dto.end_date, dto.assignee_id,
        )
        .await
        .map_err(AppError::from)
    }

    pub async fn wbs_tree(pool: &SqlitePool, project_id: i64) -> Result<Vec<WbsElement>, AppError> {
        WbsRepo::list(pool, project_id).await.map_err(AppError::from)
    }

    pub async fn update_wbs_progress(
        pool: &SqlitePool,
        project_id: i64,
        id: i64,
        dto: &UpdateWbsProgressRequest,
    ) -> Result<WbsElement, AppError> {
        if dto.progress_pct < 0.0 || dto.progress_pct > 100.0 {
            return Err(AppError::Validation("Progress must be 0-100".into()));
        }
        WbsRepo::update_progress(pool, project_id, id, dto.progress_pct)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("WBS element not found: {}", id)))
    }

    pub async fn create_transaction(
        pool: &SqlitePool,
        tenant_id: i64,
        project_id: i64,
        dto: &CreateTransactionRequest,
        created_by: Option<i64>,
    ) -> Result<ProjectTransaction, AppError> {
        if !matches!(dto.tx_type.as_str(), "budget" | "expense" | "revenue") {
            return Err(AppError::Validation(format!("Invalid transaction type: {}", dto.tx_type)));
        }
        Self::get_project(pool, tenant_id, project_id).await?;
        let tx_date = dto.tx_date.unwrap_or_else(|| chrono::Local::now().date_naive());
        ProjectTxRepo::create(
            pool, tenant_id, project_id, &dto.tx_type, dto.amount,
            dto.description.as_deref(), tx_date, created_by,
        )
        .await
        .map_err(AppError::from)
    }

    pub async fn list_transactions(pool: &SqlitePool, project_id: i64) -> Result<Vec<ProjectTransaction>, AppError> {
        ProjectTxRepo::list(pool, project_id).await.map_err(AppError::from)
    }

    pub async fn financials(pool: &SqlitePool, tenant_id: i64, project_id: i64) -> Result<ProjectFinancials, AppError> {
        ProjectRepo::financials(pool, tenant_id, project_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Project not found: {}", project_id)))
    }
}

/// Per-table sequence helper for document numbers.
async fn seq(pool: &SqlitePool, table: &str) -> Result<i64, AppError> {
    let n: i64 = sqlx::query_scalar(&format!("SELECT COALESCE(MAX(id), 0) + 1 FROM {}", table))
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(n)
}
