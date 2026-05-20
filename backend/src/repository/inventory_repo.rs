use sqlx::{SqlitePool, Transaction};
use crate::domain::{
    InboundRecord, InboundItem, OutboundRecord, OutboundItem,
    InventoryCheck, InventoryCheckItem,
};
use crate::error::{AppError, AppResult};

// ---------------------------------------------------------------------------
// InboundRepo
// ---------------------------------------------------------------------------
pub struct InboundRepo {
    db: SqlitePool,
}

impl InboundRepo {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Insert the inbound record (within an active transaction).
    pub async fn create_inbound_tx(
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        record: &InboundRecord,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO inbound_records (id, inbound_no, inbound_type, supplier_id, \
             order_id, operator_id, total_items, notes, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(&record.id)
        .bind(&record.inbound_no)
        .bind(&record.inbound_type)
        .bind(&record.supplier_id)
        .bind(&record.order_id)
        .bind(&record.operator_id)
        .bind(record.total_items)
        .bind(&record.notes)
        .bind(&record.created_at)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Insert an inbound item (within an active transaction).
    pub async fn create_inbound_item_tx(
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        item: &InboundItem,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO inbound_items (id, inbound_id, pipe_type, pipe_id, confirmed, notes) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(&item.id)
        .bind(&item.inbound_id)
        .bind(&item.pipe_type)
        .bind(&item.pipe_id)
        .bind(item.confirmed)
        .bind(&item.notes)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> AppResult<InboundRecord> {
        sqlx::query_as::<_, InboundRecord>(
            "SELECT id, inbound_no, inbound_type, supplier_id, order_id, \
             operator_id, total_items, notes, created_at \
             FROM inbound_records WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Inbound record {} not found", id)))
    }

    pub async fn find_items_by_inbound_id(&self, inbound_id: &str) -> AppResult<Vec<InboundItem>> {
        let items = sqlx::query_as::<_, InboundItem>(
            "SELECT id, inbound_id, pipe_type, pipe_id, confirmed, notes \
             FROM inbound_items WHERE inbound_id = ?1",
        )
        .bind(inbound_id)
        .fetch_all(&self.db)
        .await?;
        Ok(items)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list(
        &self,
        page: i64,
        page_size: i64,
        inbound_type: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
        supplier_id: Option<&str>,
    ) -> AppResult<(Vec<InboundRecord>, i64)> {
        let mut where_clauses: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        if let Some(t) = inbound_type {
            params.push(t.to_string());
            where_clauses.push(format!("inbound_type = ?{}", params.len()));
        }
        if let Some(s) = start_date {
            params.push(s.to_string());
            where_clauses.push(format!("created_at >= ?{}", params.len()));
        }
        if let Some(e) = end_date {
            params.push(e.to_string());
            where_clauses.push(format!("created_at <= ?{}", params.len()));
        }
        if let Some(s) = supplier_id {
            params.push(s.to_string());
            where_clauses.push(format!("supplier_id = ?{}", params.len()));
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) FROM inbound_records {}", where_sql);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        for p in &params {
            count_query = count_query.bind(p);
        }
        let total = count_query.fetch_one(&self.db).await?;

        let offset = (page - 1) * page_size;
        let query_sql = format!(
            "SELECT id, inbound_no, inbound_type, supplier_id, order_id, \
             operator_id, total_items, notes, created_at \
             FROM inbound_records {} ORDER BY created_at DESC LIMIT ?{} OFFSET ?{}",
            where_sql,
            params.len() + 1,
            params.len() + 2,
        );
        let mut query = sqlx::query_as::<_, InboundRecord>(&query_sql);
        for p in &params {
            query = query.bind(p);
        }
        let rows = query.bind(page_size).bind(offset).fetch_all(&self.db).await?;

        Ok((rows, total))
    }

    pub async fn update(&self, id: &str, inbound_type: &str, supplier_id: Option<&str>,
        order_id: Option<&str>, notes: Option<&str>, total_items: i32) -> AppResult<InboundRecord> {
        sqlx::query(
            "UPDATE inbound_records SET inbound_type = ?1, supplier_id = ?2, order_id = ?3, \
             notes = ?4, total_items = ?5 WHERE id = ?6",
        )
        .bind(inbound_type)
        .bind(supplier_id)
        .bind(order_id)
        .bind(notes)
        .bind(total_items)
        .bind(id)
        .execute(&self.db)
        .await?;
        self.find_by_id(id).await
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        // Delete items first, then the record
        sqlx::query("DELETE FROM inbound_items WHERE inbound_id = ?1")
            .bind(id)
            .execute(&self.db)
            .await?;
        sqlx::query("DELETE FROM inbound_records WHERE id = ?1")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Count how many inbound records share the same date prefix for sequence generation.
    pub async fn count_today_inbound(&self, date_prefix: &str) -> AppResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM inbound_records WHERE inbound_no LIKE ?1",
        )
        .bind(format!("{}%", date_prefix))
        .fetch_one(&self.db)
        .await?;
        Ok(count)
    }
}

// ---------------------------------------------------------------------------
// OutboundRepo
// ---------------------------------------------------------------------------
pub struct OutboundRepo {
    db: SqlitePool,
}

impl OutboundRepo {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn create_outbound_tx(
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        record: &OutboundRecord,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO outbound_records (id, outbound_no, outbound_type, customer_id, \
             order_id, operator_id, total_items, notes, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(&record.id)
        .bind(&record.outbound_no)
        .bind(&record.outbound_type)
        .bind(&record.customer_id)
        .bind(&record.order_id)
        .bind(&record.operator_id)
        .bind(record.total_items)
        .bind(&record.notes)
        .bind(&record.created_at)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn create_outbound_item_tx(
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        item: &OutboundItem,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO outbound_items (id, outbound_id, pipe_type, pipe_id, confirmed, notes) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(&item.id)
        .bind(&item.outbound_id)
        .bind(&item.pipe_type)
        .bind(&item.pipe_id)
        .bind(item.confirmed)
        .bind(&item.notes)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> AppResult<OutboundRecord> {
        sqlx::query_as::<_, OutboundRecord>(
            "SELECT id, outbound_no, outbound_type, customer_id, order_id, \
             operator_id, total_items, notes, created_at \
             FROM outbound_records WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Outbound record {} not found", id)))
    }

    pub async fn find_items_by_outbound_id(&self, outbound_id: &str) -> AppResult<Vec<OutboundItem>> {
        let items = sqlx::query_as::<_, OutboundItem>(
            "SELECT id, outbound_id, pipe_type, pipe_id, confirmed, notes \
             FROM outbound_items WHERE outbound_id = ?1",
        )
        .bind(outbound_id)
        .fetch_all(&self.db)
        .await?;
        Ok(items)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list(
        &self,
        page: i64,
        page_size: i64,
        outbound_type: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
        customer_id: Option<&str>,
    ) -> AppResult<(Vec<OutboundRecord>, i64)> {
        let mut where_clauses: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        if let Some(t) = outbound_type {
            params.push(t.to_string());
            where_clauses.push(format!("outbound_type = ?{}", params.len()));
        }
        if let Some(s) = start_date {
            params.push(s.to_string());
            where_clauses.push(format!("created_at >= ?{}", params.len()));
        }
        if let Some(e) = end_date {
            params.push(e.to_string());
            where_clauses.push(format!("created_at <= ?{}", params.len()));
        }
        if let Some(c) = customer_id {
            params.push(c.to_string());
            where_clauses.push(format!("customer_id = ?{}", params.len()));
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) FROM outbound_records {}", where_sql);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        for p in &params {
            count_query = count_query.bind(p);
        }
        let total = count_query.fetch_one(&self.db).await?;

        let offset = (page - 1) * page_size;
        let query_sql = format!(
            "SELECT id, outbound_no, outbound_type, customer_id, order_id, \
             operator_id, total_items, notes, created_at \
             FROM outbound_records {} ORDER BY created_at DESC LIMIT ?{} OFFSET ?{}",
            where_sql,
            params.len() + 1,
            params.len() + 2,
        );
        let mut query = sqlx::query_as::<_, OutboundRecord>(&query_sql);
        for p in &params {
            query = query.bind(p);
        }
        let rows = query.bind(page_size).bind(offset).fetch_all(&self.db).await?;

        Ok((rows, total))
    }

    pub async fn update(&self, id: &str, outbound_type: &str, customer_id: Option<&str>,
        order_id: Option<&str>, notes: Option<&str>, total_items: i32) -> AppResult<OutboundRecord> {
        sqlx::query(
            "UPDATE outbound_records SET outbound_type = ?1, customer_id = ?2, order_id = ?3, \
             notes = ?4, total_items = ?5 WHERE id = ?6",
        )
        .bind(outbound_type)
        .bind(customer_id)
        .bind(order_id)
        .bind(notes)
        .bind(total_items)
        .bind(id)
        .execute(&self.db)
        .await?;
        self.find_by_id(id).await
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM outbound_items WHERE outbound_id = ?1")
            .bind(id)
            .execute(&self.db)
            .await?;
        sqlx::query("DELETE FROM outbound_records WHERE id = ?1")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn count_today_outbound(&self, date_prefix: &str) -> AppResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM outbound_records WHERE outbound_no LIKE ?1",
        )
        .bind(format!("{}%", date_prefix))
        .fetch_one(&self.db)
        .await?;
        Ok(count)
    }
}

// ---------------------------------------------------------------------------
// StockRepo
// ---------------------------------------------------------------------------
pub struct StockRepo {
    db: SqlitePool,
}

impl StockRepo {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Total count of in_stock pipes across both tables.
    pub async fn total_in_stock(&self) -> AppResult<i64> {
        let s: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM seamless_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        )
        .fetch_one(&self.db)
        .await?;
        let sc: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM screen_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        )
        .fetch_one(&self.db)
        .await?;
        Ok(s + sc)
    }

    /// Total seamless pipes by status.
    pub async fn seamless_by_status(&self) -> AppResult<Vec<(String, i64)>> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT status, COUNT(*) as cnt FROM seamless_pipes \
             WHERE deleted_at IS NULL GROUP BY status",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    /// Total screen pipes by status.
    pub async fn screen_by_status(&self) -> AppResult<Vec<(String, i64)>> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT status, COUNT(*) as cnt FROM screen_pipes \
             WHERE deleted_at IS NULL GROUP BY status",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    pub async fn count_seamless_pipes(&self) -> AppResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM seamless_pipes WHERE deleted_at IS NULL",
        )
        .fetch_one(&self.db)
        .await?;
        Ok(count)
    }

    pub async fn count_screen_pipes(&self) -> AppResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM screen_pipes WHERE deleted_at IS NULL",
        )
        .fetch_one(&self.db)
        .await?;
        Ok(count)
    }

    /// Group in_stock pipes by grade.
    pub async fn stock_by_grade(&self) -> AppResult<Vec<(String, i64, i64)>> {
        let seamless: Vec<(String, i64)> = sqlx::query_as(
            "SELECT grade, COUNT(*) as cnt FROM seamless_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL GROUP BY grade ORDER BY grade",
        )
        .fetch_all(&self.db)
        .await?;

        let screen: Vec<(String, i64)> = sqlx::query_as(
            "SELECT grade, COUNT(*) as cnt FROM screen_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL GROUP BY grade ORDER BY grade",
        )
        .fetch_all(&self.db)
        .await?;

        // Merge by grade
        use std::collections::BTreeMap;
        let mut map: BTreeMap<String, (i64, i64)> = BTreeMap::new();
        for (g, c) in seamless {
            map.entry(g).or_insert((0, 0)).0 = c;
        }
        for (g, c) in screen {
            map.entry(g).or_insert((0, 0)).1 = c;
        }
        Ok(map.into_iter().map(|(k, (s, sc))| (k, s, sc)).collect())
    }

    /// Group in_stock pipes by location.
    pub async fn stock_by_location(&self) -> AppResult<Vec<(String, i64, i64)>> {
        let seamless: Vec<(Option<String>, i64)> = sqlx::query_as(
            "SELECT location, COUNT(*) as cnt FROM seamless_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL GROUP BY location ORDER BY location",
        )
        .fetch_all(&self.db)
        .await?;

        let screen: Vec<(Option<String>, i64)> = sqlx::query_as(
            "SELECT location, COUNT(*) as cnt FROM screen_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL GROUP BY location ORDER BY location",
        )
        .fetch_all(&self.db)
        .await?;

        use std::collections::BTreeMap;
        let mut map: BTreeMap<String, (i64, i64)> = BTreeMap::new();
        for (loc, c) in seamless {
            let key = loc.unwrap_or_else(|| "unknown".to_string());
            map.entry(key).or_insert((0, 0)).0 = c;
        }
        for (loc, c) in screen {
            let key = loc.unwrap_or_else(|| "unknown".to_string());
            map.entry(key).or_insert((0, 0)).1 = c;
        }
        Ok(map.into_iter().map(|(k, (s, sc))| (k, s, sc)).collect())
    }

    /// Validate that a pipe exists and is in_stock (for outbound).
    pub async fn validate_pipe_in_stock(&self, pipe_type: &str, pipe_id: &str) -> AppResult<bool> {
        let exists = match pipe_type {
            "seamless" => {
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM seamless_pipes WHERE id = ?1 AND status = 'in_stock' AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_one(&self.db)
                .await?
            }
            "screen" => {
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM screen_pipes WHERE id = ?1 AND status = 'in_stock' AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_one(&self.db)
                .await?
            }
            _ => return Err(AppError::BadRequest(format!("Invalid pipe_type: {}", pipe_type))),
        };
        Ok(exists > 0)
    }

    /// Validate that a pipe exists (for inbound).
    pub async fn validate_pipe_exists(&self, pipe_type: &str, pipe_id: &str) -> AppResult<bool> {
        let exists = match pipe_type {
            "seamless" => {
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM seamless_pipes WHERE id = ?1 AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_one(&self.db)
                .await?
            }
            "screen" => {
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM screen_pipes WHERE id = ?1 AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_one(&self.db)
                .await?
            }
            _ => return Err(AppError::BadRequest(format!("Invalid pipe_type: {}", pipe_type))),
        };
        Ok(exists > 0)
    }
}

// ---------------------------------------------------------------------------
// InventoryCheckRepo
// ---------------------------------------------------------------------------
pub struct InventoryCheckRepo {
    db: SqlitePool,
}

impl InventoryCheckRepo {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn create_check_tx(
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        check: &InventoryCheck,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO inventory_checks (id, check_no, check_type, operator_id, \
             total_expected, total_confirmed, total_missing, notes, status, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        )
        .bind(&check.id)
        .bind(&check.check_no)
        .bind(&check.check_type)
        .bind(&check.operator_id)
        .bind(check.total_expected)
        .bind(check.total_confirmed)
        .bind(check.total_missing)
        .bind(&check.notes)
        .bind(&check.status)
        .bind(&check.created_at)
        .bind(&check.updated_at)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn create_check_item_tx(
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        item: &InventoryCheckItem,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO inventory_check_items (id, check_id, pipe_type, pipe_id, expected, confirmed, notes) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .bind(&item.id)
        .bind(&item.check_id)
        .bind(&item.pipe_type)
        .bind(&item.pipe_id)
        .bind(item.expected)
        .bind(item.confirmed)
        .bind(&item.notes)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> AppResult<InventoryCheck> {
        sqlx::query_as::<_, InventoryCheck>(
            "SELECT id, check_no, check_type, operator_id, total_expected, total_confirmed, \
             total_missing, notes, status, created_at, updated_at \
             FROM inventory_checks WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Inventory check {} not found", id)))
    }

    pub async fn find_items_by_check_id(&self, check_id: &str) -> AppResult<Vec<InventoryCheckItem>> {
        let items = sqlx::query_as::<_, InventoryCheckItem>(
            "SELECT id, check_id, pipe_type, pipe_id, expected, confirmed, notes \
             FROM inventory_check_items WHERE check_id = ?1",
        )
        .bind(check_id)
        .fetch_all(&self.db)
        .await?;
        Ok(items)
    }

    pub async fn list(
        &self,
        page: i64,
        page_size: i64,
        status: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> AppResult<(Vec<InventoryCheck>, i64)> {
        let mut where_clauses: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        if let Some(s) = status {
            params.push(s.to_string());
            where_clauses.push(format!("status = ?{}", params.len()));
        }
        if let Some(s) = start_date {
            params.push(s.to_string());
            where_clauses.push(format!("created_at >= ?{}", params.len()));
        }
        if let Some(e) = end_date {
            params.push(e.to_string());
            where_clauses.push(format!("created_at <= ?{}", params.len()));
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) FROM inventory_checks {}", where_sql);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        for p in &params {
            count_query = count_query.bind(p);
        }
        let total = count_query.fetch_one(&self.db).await?;

        let offset = (page - 1) * page_size;
        let query_sql = format!(
            "SELECT id, check_no, check_type, operator_id, total_expected, total_confirmed, \
             total_missing, notes, status, created_at, updated_at \
             FROM inventory_checks {} ORDER BY created_at DESC LIMIT ?{} OFFSET ?{}",
            where_sql,
            params.len() + 1,
            params.len() + 2,
        );
        let mut query = sqlx::query_as::<_, InventoryCheck>(&query_sql);
        for p in &params {
            query = query.bind(p);
        }
        let rows = query.bind(page_size).bind(offset).fetch_all(&self.db).await?;

        Ok((rows, total))
    }

    pub async fn count_today_checks(&self, date_prefix: &str) -> AppResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM inventory_checks WHERE check_no LIKE ?1",
        )
        .bind(format!("{}%", date_prefix))
        .fetch_one(&self.db)
        .await?;
        Ok(count)
    }
}
