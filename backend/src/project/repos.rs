//! Project repositories.

use sqlx::SqlitePool;
use crate::models::project::{Project, ProjectFinancials, ProjectTransaction, WbsElement};

pub struct ProjectRepo;

impl ProjectRepo {
    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        project_no: &str,
        name: &str,
        description: Option<&str>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        manager_id: Option<i64>,
        budget: f64,
    ) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "INSERT INTO projects \
             (tenant_id, project_no, name, description, start_date, end_date, manager_id, budget) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
             RETURNING id, tenant_id, project_no, name, description, status, start_date, \
                       end_date, manager_id, budget, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(project_no)
        .bind(name)
        .bind(description)
        .bind(start_date)
        .bind(end_date)
        .bind(manager_id)
        .bind(budget)
        .fetch_one(pool)
        .await
    }

    pub async fn list(pool: &SqlitePool, tenant_id: i64, status: Option<&str>) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, tenant_id, project_no, name, description, status, start_date, \
                    end_date, manager_id, budget, created_at, updated_at, deleted_at \
             FROM projects WHERE tenant_id = ? AND deleted_at IS NULL \
             AND (? IS NULL OR status = ?) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(status)
        .bind(status)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, tenant_id, project_no, name, description, status, start_date, \
                    end_date, manager_id, budget, created_at, updated_at, deleted_at \
             FROM projects WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_status(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        status: &str,
    ) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "UPDATE projects SET status = ?, updated_at = datetime('now') \
             WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL \
             RETURNING id, tenant_id, project_no, name, description, status, start_date, \
                       end_date, manager_id, budget, created_at, updated_at, deleted_at",
        )
        .bind(status)
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn financials(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<Option<ProjectFinancials>, sqlx::Error> {
        sqlx::query_as::<_, ProjectFinancials>(
            "SELECT p.id AS project_id, p.budget, \
                    CAST(COALESCE((SELECT SUM(t.amount) FROM project_transactions t \
                              WHERE t.project_id = p.id AND t.tx_type = 'expense'), 0.0) AS REAL) AS expense_total, \
                    CAST(COALESCE((SELECT SUM(t.amount) FROM project_transactions t \
                              WHERE t.project_id = p.id AND t.tx_type = 'revenue'), 0.0) AS REAL) AS revenue_total, \
                    (p.budget - CAST(COALESCE((SELECT SUM(t.amount) FROM project_transactions t \
                              WHERE t.project_id = p.id AND t.tx_type = 'expense'), 0.0) AS REAL)) AS remaining \
             FROM projects p WHERE p.tenant_id = ? AND p.id = ? AND p.deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

pub struct WbsRepo;

impl WbsRepo {
    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        project_id: i64,
        parent_id: Option<i64>,
        code: &str,
        name: &str,
        weight_pct: Option<f64>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        assignee_id: Option<i64>,
    ) -> Result<WbsElement, sqlx::Error> {
        sqlx::query_as::<_, WbsElement>(
            "INSERT INTO wbs_elements \
             (tenant_id, project_id, parent_id, code, name, weight_pct, start_date, end_date, assignee_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) \
             RETURNING id, tenant_id, project_id, parent_id, code, name, sort_order, \
                       weight_pct, progress_pct, start_date, end_date, assignee_id, created_at, updated_at",
        )
        .bind(tenant_id)
        .bind(project_id)
        .bind(parent_id)
        .bind(code)
        .bind(name)
        .bind(weight_pct)
        .bind(start_date)
        .bind(end_date)
        .bind(assignee_id)
        .fetch_one(pool)
        .await
    }

    pub async fn list(pool: &SqlitePool, project_id: i64) -> Result<Vec<WbsElement>, sqlx::Error> {
        sqlx::query_as::<_, WbsElement>(
            "SELECT id, tenant_id, project_id, parent_id, code, name, sort_order, \
                    weight_pct, progress_pct, start_date, end_date, assignee_id, created_at, updated_at \
             FROM wbs_elements WHERE project_id = ? ORDER BY sort_order, id",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
    }

    pub async fn update_progress(
        pool: &SqlitePool,
        project_id: i64,
        id: i64,
        progress_pct: f64,
    ) -> Result<Option<WbsElement>, sqlx::Error> {
        sqlx::query_as::<_, WbsElement>(
            "UPDATE wbs_elements SET progress_pct = ?, updated_at = datetime('now') \
             WHERE project_id = ? AND id = ? \
             RETURNING id, tenant_id, project_id, parent_id, code, name, sort_order, \
                       weight_pct, progress_pct, start_date, end_date, assignee_id, created_at, updated_at",
        )
        .bind(progress_pct)
        .bind(project_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

pub struct ProjectTxRepo;

impl ProjectTxRepo {
    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        project_id: i64,
        tx_type: &str,
        amount: f64,
        description: Option<&str>,
        tx_date: chrono::NaiveDate,
        created_by: Option<i64>,
    ) -> Result<ProjectTransaction, sqlx::Error> {
        sqlx::query_as::<_, ProjectTransaction>(
            "INSERT INTO project_transactions \
             (tenant_id, project_id, tx_type, amount, description, tx_date, created_by) \
             VALUES (?, ?, ?, ?, ?, ?, ?) \
             RETURNING id, tenant_id, project_id, tx_type, amount, description, tx_date, \
                       created_by, created_at",
        )
        .bind(tenant_id)
        .bind(project_id)
        .bind(tx_type)
        .bind(amount)
        .bind(description)
        .bind(tx_date)
        .bind(created_by)
        .fetch_one(pool)
        .await
    }

    pub async fn list(pool: &SqlitePool, project_id: i64) -> Result<Vec<ProjectTransaction>, sqlx::Error> {
        sqlx::query_as::<_, ProjectTransaction>(
            "SELECT id, tenant_id, project_id, tx_type, amount, description, tx_date, \
                    created_by, created_at \
             FROM project_transactions WHERE project_id = ? ORDER BY tx_date DESC, id DESC LIMIT 500",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
    }
}
