//! Project repositories.

use sqlx::PgPool;
use crate::models::project::{Project, ProjectFinancials, ProjectTransaction, WbsElement};

pub struct ProjectRepo;

impl ProjectRepo {
    pub async fn create(
        pool: &PgPool,
        tenant_id: i64,
        project_no: &str,
        name: &str,
        description: Option<&str>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        manager_id: Option<i64>,
        budget: rust_decimal::Decimal,
    ) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "INSERT INTO projects \
             (tenant_id, project_no, name, description, start_date, end_date, manager_id, budget) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
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

    pub async fn list(pool: &PgPool, tenant_id: i64, status: Option<&str>) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, tenant_id, project_no, name, description, status, start_date, \
                    end_date, manager_id, budget, created_at, updated_at, deleted_at \
             FROM projects WHERE tenant_id = $1 AND deleted_at IS NULL \
             AND ($2::text IS NULL OR status = $2) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(status)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, tenant_id: i64, id: i64) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, tenant_id, project_no, name, description, status, start_date, \
                    end_date, manager_id, budget, created_at, updated_at, deleted_at \
             FROM projects WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_status(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
        status: &str,
    ) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "UPDATE projects SET status = $3, updated_at = NOW() \
             WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL \
             RETURNING id, tenant_id, project_no, name, description, status, start_date, \
                       end_date, manager_id, budget, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(id)
        .bind(status)
        .fetch_optional(pool)
        .await
    }

    pub async fn financials(pool: &PgPool, tenant_id: i64, id: i64) -> Result<Option<ProjectFinancials>, sqlx::Error> {
        sqlx::query_as::<_, ProjectFinancials>(
            "SELECT p.id AS project_id, p.budget, \
                    COALESCE((SELECT SUM(t.amount) FROM project_transactions t \
                              WHERE t.project_id = p.id AND t.tx_type = 'expense'), 0) AS expense_total, \
                    COALESCE((SELECT SUM(t.amount) FROM project_transactions t \
                              WHERE t.project_id = p.id AND t.tx_type = 'revenue'), 0) AS revenue_total, \
                    (p.budget - COALESCE((SELECT SUM(t.amount) FROM project_transactions t \
                              WHERE t.project_id = p.id AND t.tx_type = 'expense'), 0)) AS remaining \
             FROM projects p WHERE p.tenant_id = $1 AND p.id = $2 AND p.deleted_at IS NULL",
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
        pool: &PgPool,
        tenant_id: i64,
        project_id: i64,
        parent_id: Option<i64>,
        code: &str,
        name: &str,
        weight_pct: Option<rust_decimal::Decimal>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        assignee_id: Option<i64>,
    ) -> Result<WbsElement, sqlx::Error> {
        sqlx::query_as::<_, WbsElement>(
            "INSERT INTO wbs_elements \
             (tenant_id, project_id, parent_id, code, name, weight_pct, start_date, end_date, assignee_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
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

    pub async fn list(pool: &PgPool, project_id: i64) -> Result<Vec<WbsElement>, sqlx::Error> {
        sqlx::query_as::<_, WbsElement>(
            "SELECT id, tenant_id, project_id, parent_id, code, name, sort_order, \
                    weight_pct, progress_pct, start_date, end_date, assignee_id, created_at, updated_at \
             FROM wbs_elements WHERE project_id = $1 ORDER BY sort_order, id",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
    }

    pub async fn update_progress(
        pool: &PgPool,
        project_id: i64,
        id: i64,
        progress_pct: rust_decimal::Decimal,
    ) -> Result<Option<WbsElement>, sqlx::Error> {
        sqlx::query_as::<_, WbsElement>(
            "UPDATE wbs_elements SET progress_pct = $3, updated_at = NOW() \
             WHERE project_id = $1 AND id = $2 \
             RETURNING id, tenant_id, project_id, parent_id, code, name, sort_order, \
                       weight_pct, progress_pct, start_date, end_date, assignee_id, created_at, updated_at",
        )
        .bind(project_id)
        .bind(id)
        .bind(progress_pct)
        .fetch_optional(pool)
        .await
    }
}

pub struct ProjectTxRepo;

impl ProjectTxRepo {
    pub async fn create(
        pool: &PgPool,
        tenant_id: i64,
        project_id: i64,
        tx_type: &str,
        amount: rust_decimal::Decimal,
        description: Option<&str>,
        tx_date: chrono::NaiveDate,
        created_by: Option<i64>,
    ) -> Result<ProjectTransaction, sqlx::Error> {
        sqlx::query_as::<_, ProjectTransaction>(
            "INSERT INTO project_transactions \
             (tenant_id, project_id, tx_type, amount, description, tx_date, created_by) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
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

    pub async fn list(pool: &PgPool, project_id: i64) -> Result<Vec<ProjectTransaction>, sqlx::Error> {
        sqlx::query_as::<_, ProjectTransaction>(
            "SELECT id, tenant_id, project_id, tx_type, amount, description, tx_date, \
                    created_by, created_at \
             FROM project_transactions WHERE project_id = $1 ORDER BY tx_date DESC, id DESC LIMIT 500",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
    }
}
