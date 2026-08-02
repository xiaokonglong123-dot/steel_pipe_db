use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::dto::common::PaginationParams;
use crate::dto::pipe_dto::PipeFilterParams;
use crate::domain::pipe::{PipeModel, PipeStatus, PipeType};
use crate::error::AppError;

pub struct GenericPipeRepo<P: PipeModel> {
    _phantom: std::marker::PhantomData<P>,
}

impl<P: PipeModel> GenericPipeRepo<P> {
    pub async fn create(
        pool: &SqlitePool,
        dto: &P::CreateDto,
    ) -> Result<P, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(format!(
            "INSERT INTO {} (pipe_number, batch_number",
            P::TABLE_NAME
        ));
        P::build_create_query(&mut builder, dto);
        builder.push(") RETURNING *");
        
        builder.build_query_as::<P>().fetch_one(pool).await
    }

    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        dto: &P::UpdateDto,
    ) -> Result<P, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(format!(
            "UPDATE {} SET ",
            P::TABLE_NAME
        ));
        P::build_update_query(&mut builder, dto);
        builder.push(" WHERE id = ").push_bind(id);
        builder.push(" AND deleted_at IS NULL RETURNING *");
        
        builder.build_query_as::<P>().fetch_one(pool).await
    }

    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<P>, sqlx::Error> {
        let query = format!(
            "SELECT * FROM {} WHERE id = ? AND deleted_at IS NULL",
            P::TABLE_NAME
        );
        sqlx::query_as::<_, P>(&query)
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_ids(
        pool: &SqlitePool,
        ids: &[i64],
    ) -> Result<Vec<P>, sqlx::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT * FROM {} WHERE id IN ({}) AND deleted_at IS NULL",
            P::TABLE_NAME,
            placeholders.join(",")
        );
        let mut q = sqlx::query_as::<_, P>(&query);
        for id in ids {
            q = q.bind(id);
        }
        q.fetch_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(&format!(
            "UPDATE {} SET deleted_at = datetime('now'), updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
            P::TABLE_NAME
        ))
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_pipe_number(
        pool: &SqlitePool,
        pipe_number: &str,
    ) -> Result<Option<P>, sqlx::Error> {
        let query = format!(
            "SELECT * FROM {} WHERE pipe_number = ? AND deleted_at IS NULL",
            P::TABLE_NAME
        );
        sqlx::query_as::<_, P>(&query)
            .bind(pipe_number)
            .fetch_optional(pool)
            .await
    }

    pub async fn list(
        pool: &SqlitePool,
        filter: &PipeFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<P>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        let mut conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push("(pipe_number LIKE ? OR batch_number LIKE ?)".into());
                let pattern = format!("%{}%", q);
                bind_values.push(pattern.clone());
                bind_values.push(pattern);
            }
        }

        if let Some(ref grade) = filter.grade {
            conditions.push(format!("{} = ?", P::GRADE_COLUMN));
            bind_values.push(grade.clone());
        }

        if let Some(ref pipe_type) = filter.pipe_type {
            // Skip the filter when the value names the pipe class itself
            // (e.g. "seamless" against seamless_pipes): the sub-type column
            // (pipe_type/screen_type) stores sub-types like casing/tubing,
            // not classes. Only real sub-types become column filters.
            let is_class = PipeType::from_pipe_type_str(pipe_type) == Some(P::PIPE_TYPE);
            if !is_class {
                conditions.push(format!("{} = ?", P::PIPE_TYPE_COLUMN));
                bind_values.push(pipe_type.clone());
            }
        }

        if let Some(ref status) = filter.status {
            conditions.push("status = ?".into());
            bind_values.push(status.clone());
        }

        if let Some(od_min) = filter.od_min {
            conditions.push(format!("{} >= ?", P::OD_COLUMN));
            bind_values.push(od_min.to_string());
        }
        if let Some(od_max) = filter.od_max {
            conditions.push(format!("{} <= ?", P::OD_COLUMN));
            bind_values.push(od_max.to_string());
        }

        if let Some(wt_min) = filter.wt_min {
            conditions.push(format!("{} >= ?", P::WT_COLUMN));
            bind_values.push(wt_min.to_string());
        }
        if let Some(wt_max) = filter.wt_max {
            conditions.push(format!("{} <= ?", P::WT_COLUMN));
            bind_values.push(wt_max.to_string());
        }

        if let Some(location_id) = filter.location_id {
            conditions.push("location_id = ?".into());
            bind_values.push(location_id.to_string());
        }

        if let Some(ref manufacturer) = filter.manufacturer {
            conditions.push("manufacturer = ?".into());
            bind_values.push(manufacturer.clone());
        }

        if let Some(ref heat_number) = filter.heat_number {
            conditions.push("heat_number = ?".into());
            bind_values.push(heat_number.clone());
        }

        let where_clause = conditions.join(" AND ");

        let sort_col = params
            .sort_by
            .as_deref()
            .and_then(|col| P::valid_sort_column(col))
            .unwrap_or("created_at");
        let sort_order = params.sort_order_sql();

        let count_sql = format!("SELECT COUNT(*) as cnt FROM {} WHERE {}", P::TABLE_NAME, where_clause);
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT * FROM {} WHERE {} ORDER BY {} {} LIMIT ? OFFSET ?",
            P::TABLE_NAME, where_clause, sort_col, sort_order
        );
        let mut list_q = sqlx::query_as::<_, P>(&list_sql);
        for val in &bind_values {
            list_q = list_q.bind(val.as_str());
        }
        let items = list_q
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await?;

        Ok((items, total.0 as u64))
    }

    pub async fn search(pool: &SqlitePool, query: &str) -> Result<Vec<P>, sqlx::Error> {
        let like = format!("%{}%", query);
        let search_sql = format!(
            "SELECT * FROM {} WHERE deleted_at IS NULL AND (pipe_number LIKE ? OR batch_number LIKE ?) ORDER BY created_at DESC LIMIT 50",
            P::TABLE_NAME
        );
        sqlx::query_as::<_, P>(&search_sql)
            .bind(&like)
            .bind(&like)
            .fetch_all(pool)
            .await
    }

    pub fn validate_status_transition(current: &str, target: &str) -> Result<(), AppError> {
        let current_status = PipeStatus::from_str(current)
            .ok_or_else(|| AppError::Validation(format!("Invalid current status: {}", current)))?;
        let target_status = PipeStatus::from_str(target)
            .ok_or_else(|| AppError::Validation(format!("Invalid target status: {}", target)))?;

        if !current_status.can_transition_to(target_status) {
            return Err(AppError::Validation(format!(
                "Invalid status transition from '{}' to '{}'",
                current, target
            )));
        }
        Ok(())
    }
}