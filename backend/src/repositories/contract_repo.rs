use sqlx::{postgres::SqliteConnection, QueryBuilder, Sqlite, PgPool};

use crate::domain::money::{from_decimal, from_decimal_opt};
use crate::dto::common::PaginationParams;
use crate::dto::contract_dto::{
    ContractFilterParams, CreateContractItemRequest, CreateContractRequest, CreatePaymentRequest,
    UpdateContractItemRequest, UpdateContractRequest, UpdatePaymentRequest,
};
use crate::models::contract::{Contract, ContractItem, ContractPayment};

/// CRUD for `contracts`, `contract_items`, and `contract_payments`.
/// All contract queries filter `deleted_at IS NULL`.
pub struct ContractRepo;

impl ContractRepo {
    /// Generated the next sequential contract number (`CT-SAL-000001` / `CT-PUR-000001`).
    async fn next_contract_no(
        pool: &PgPool,
        contract_type: &str,
    ) -> Result<String, sqlx::Error> {
        let prefix = match contract_type {
            "sales" => "CT-SAL",
            "purchase" => "CT-PUR",
            _ => "CT",
        };
        let like = format!("{}%", prefix);
        let row: (Option<String>,) =
            sqlx::query_as("SELECT MAX(contract_no) FROM contracts WHERE contract_no LIKE $1")
                .bind(&like)
                .fetch_optional(pool)
                .await?
                .unwrap_or((None,));

        let next_seq = match row.0 {
            Some(last) => {
                let parts: Vec<&str> = last.split('-').collect();
                let num_str = parts.last().unwrap_or(&"000000");
                let num: i64 = num_str.parse().unwrap_or(0);
                num + 1
            }
            None => 1,
        };

        Ok(format!("{}-{:06}", prefix, next_seq))
    }

    /// INSERT a new contract with auto-generated `contract_no`. Status starts as `draft`.
    /// Returns the created `Contract`.
    pub async fn create(
        pool: &PgPool,
        dto: &CreateContractRequest,
    ) -> Result<Contract, sqlx::Error> {
        let contract_no = Self::next_contract_no(pool, &dto.contract_type).await?;

        sqlx::query_as::<_, Contract>(
            "INSERT INTO contracts (contract_no, contract_type, title, party_a, party_b, \
             sign_date, start_date, end_date, total_amount, status, notes) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, 'draft', $9) \
             RETURNING id, contract_no, contract_type, title, party_a, party_b, \
               sign_date, start_date, end_date, total_amount, status, notes, created_by, \
               created_at, updated_at, deleted_at",
        )
        .bind(&contract_no)
        .bind(&dto.contract_type)
        .bind(&dto.title)
        .bind(&dto.party_a)
        .bind(&dto.party_b)
        .bind(&dto.sign_date)
        .bind(&dto.start_date)
        .bind(&dto.end_date)
        .bind(&dto.notes)
        .fetch_one(pool)
        .await
    }

    /// Dynamic UPDATE of contract fields (title, party_a, party_b, dates, notes).
    /// Only supplied fields change. Returns the updated `Contract`.
    pub async fn update(
        pool: &PgPool,
        id: i64,
        dto: &UpdateContractRequest,
    ) -> Result<Contract, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE contracts SET updated_at = NOW()");

        if let Some(ref val) = dto.title {
            builder.push(", title = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.party_a {
            builder.push(", party_a = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.party_b {
            builder.push(", party_b = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.sign_date {
            builder.push(", sign_date = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.start_date {
            builder.push(", start_date = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.end_date {
            builder.push(", end_date = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.notes {
            builder.push(", notes = ");
            builder.push_bind(val);
        }

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(
            " AND deleted_at IS NULL RETURNING id, contract_no, contract_type, \
            title, party_a, party_b, sign_date, start_date, end_date, total_amount, \
            status, notes, created_by, created_at, updated_at, deleted_at",
        );

        builder.build_query_as::<Contract>().fetch_one(pool).await
    }

    /// SELECT by primary key. Returns `None` if soft-deleted or missing.
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<Contract>, sqlx::Error> {
        sqlx::query_as::<_, Contract>(
            "SELECT id, contract_no, contract_type, title, party_a, party_b, sign_date, \
             start_date, end_date, total_amount, status, notes, created_by, created_at, \
             updated_at, deleted_at \
             FROM contracts WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Soft-delete: sets `deleted_at` and `updated_at`.
    pub async fn delete(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE contracts SET deleted_at = NOW(), \
             updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// UPDATE `status`. Returns the updated `Contract`.
    pub async fn update_status(
        pool: &PgPool,
        id: i64,
        status: &str,
    ) -> Result<Contract, sqlx::Error> {
        sqlx::query_as::<_, Contract>(
            "UPDATE contracts SET status = $1, updated_at = NOW() \
             WHERE id = $2 AND deleted_at IS NULL \
             RETURNING id, contract_no, contract_type, title, party_a, party_b, sign_date, \
               start_date, end_date, total_amount, status, notes, created_by, created_at, \
               updated_at, deleted_at",
        )
        .bind(status)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// Recalculate `total_amount` from contract_items SUM. Called after item changes.
    pub async fn update_total_amount(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE contracts SET total_amount = (SELECT COALESCE(SUM(total_price), 0) \
             FROM contract_items WHERE contract_id = $1), updated_at = NOW() \
             WHERE id = $2 AND deleted_at IS NULL",
        )
        .bind(id)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Paginated SELECT with dynamic filters (q, contract_type, status, sign_date range).
    /// Supports sorting by contract_no, contract_type, title, status, sign_date, total_amount.
    /// Returns `(items, total)`.
    pub async fn list(
        pool: &PgPool,
        filter: &ContractFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<Contract>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        let mut conditions: Vec<String> = vec!["c.deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push(
                    "(c.contract_no LIKE ? OR c.title LIKE ? \
                 OR c.party_a LIKE ? OR c.party_b LIKE ?)"
                        .into(),
                );
                let pattern = format!("%{}%", q);
                bind_values.push(pattern.clone());
                bind_values.push(pattern.clone());
                bind_values.push(pattern.clone());
                bind_values.push(pattern);
            }
        }
        if let Some(ref contract_type) = filter.contract_type {
            conditions.push(format!("c.contract_type = ${}", bind_values.len() + 1));
            bind_values.push(contract_type.clone());
        }
        if let Some(ref status) = filter.status {
            conditions.push(format!("c.status = ${}", bind_values.len() + 1));
            bind_values.push(status.clone());
        }
        if let Some(ref date_from) = filter.date_from {
            conditions.push(format!("c.sign_date >= ${}", bind_values.len() + 1));
            bind_values.push(date_from.clone());
        }
        if let Some(ref date_to) = filter.date_to {
            conditions.push(format!("c.sign_date <= ${}", bind_values.len() + 1));
            bind_values.push(date_to.clone());
        }

        let where_clause = conditions.join(" AND ");

        let sort_by = match params.sort_by.as_deref() {
            Some("contract_no") => "c.contract_no",
            Some("contract_type") => "c.contract_type",
            Some("title") => "c.title",
            Some("status") => "c.status",
            Some("sign_date") => "c.sign_date",
            Some("total_amount") => "c.total_amount",
            _ => "c.created_at",
        };
        let sort_order = params.sort_order_sql();

        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM contracts c WHERE {}",
            where_clause
        );
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT c.id, c.contract_no, c.contract_type, c.title, c.party_a, c.party_b, \
             c.sign_date, c.start_date, c.end_date, c.total_amount, c.status, c.notes, \
             c.created_by, c.created_at, c.updated_at, c.deleted_at \
             FROM contracts c WHERE {} \
             ORDER BY {} {} LIMIT $1 OFFSET $2",
            where_clause, sort_by, sort_order
        );
        let mut list_q = sqlx::query_as::<_, Contract>(&list_sql);
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

    /// INSERT multiple items for a contract. Computes `total_price = quantity * unit_price`.
    /// Returns all created `ContractItem` rows.
    pub async fn create_items(
        pool: &PgPool,
        contract_id: i64,
        items: &[CreateContractItemRequest],
    ) -> Result<Vec<ContractItem>, sqlx::Error> {
        let mut results = Vec::with_capacity(items.len());
        for item in items {
            let total_price = item
                .unit_price
                .map(|p| from_decimal(p) * item.quantity as f64)
                .unwrap_or(0.0);
            let row = sqlx::query_as::<_, ContractItem>(
                "INSERT INTO contract_items (contract_id, pipe_type, grade, od, wt, \
                 quantity, unit_price, total_price, notes) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
                 RETURNING id, contract_id, pipe_type, grade, od, wt, quantity, \
                   unit_price, total_price, notes, created_at",
            )
            .bind(contract_id)
            .bind(&item.pipe_type)
            .bind(&item.grade)
            .bind(item.od)
            .bind(item.wt)
            .bind(item.quantity)
            .bind(from_decimal_opt(item.unit_price))
            .bind(total_price)
            .bind(&item.notes)
            .fetch_one(pool)
            .await?;
            results.push(row);
        }
        Ok(results)
    }

    /// SELECT items for a contract, ordered by `id`.
    pub async fn find_items_by_contract(
        pool: &PgPool,
        contract_id: i64,
    ) -> Result<Vec<ContractItem>, sqlx::Error> {
        sqlx::query_as::<_, ContractItem>(
            "SELECT id, contract_id, pipe_type, grade, od, wt, quantity, unit_price, \
             total_price, notes, created_at \
             FROM contract_items WHERE contract_id = $1 ORDER BY id",
        )
        .bind(contract_id)
        .fetch_all(pool)
        .await
    }

    /// SELECT a single item by primary key. Returns `None` if not found.
    pub async fn find_item_by_id(
        pool: &PgPool,
        item_id: i64,
    ) -> Result<Option<ContractItem>, sqlx::Error> {
        sqlx::query_as::<_, ContractItem>(
            "SELECT id, contract_id, pipe_type, grade, od, wt, quantity, unit_price, \
             total_price, notes, created_at \
             FROM contract_items WHERE id = $1",
        )
        .bind(item_id)
        .fetch_optional(pool)
        .await
    }

    /// Dynamic UPDATE of item fields. Recomputes `total_price` when `unit_price` or `quantity`
    /// changes. Returns the updated `ContractItem`.
    pub async fn update_item(
        pool: &PgPool,
        item_id: i64,
        dto: &UpdateContractItemRequest,
    ) -> Result<ContractItem, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE contract_items SET");

        let mut sep = false;
        if let Some(ref val) = dto.pipe_type {
            if sep {
                builder.push(", ");
            }
            builder.push(" pipe_type = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(ref val) = dto.grade {
            if sep {
                builder.push(", ");
            }
            builder.push(" grade = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.od {
            if sep {
                builder.push(", ");
            }
            builder.push(" od = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.wt {
            if sep {
                builder.push(", ");
            }
            builder.push(" wt = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.quantity {
            if sep {
                builder.push(", ");
            }
            builder.push(" quantity = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.unit_price {
            if sep {
                builder.push(", ");
            }
            builder.push(" unit_price = ");
            builder.push_bind(from_decimal(val));
            sep = true;
        }
        if let Some(ref val) = dto.notes {
            if sep {
                builder.push(", ");
            }
            builder.push(" notes = ");
            builder.push_bind(val);
            sep = true;
        }

        if !sep {
            return sqlx::query_as::<_, ContractItem>(
                "SELECT id, contract_id, pipe_type, grade, od, wt, quantity, unit_price, \
                 total_price, notes, created_at FROM contract_items WHERE id = $1",
            )
            .bind(item_id)
            .fetch_one(pool)
            .await;
        }

        builder.push(" WHERE id = ");
        builder.push_bind(item_id);
        builder.push(
            " RETURNING id, contract_id, pipe_type, grade, od, wt, quantity, \
            unit_price, total_price, notes, created_at",
        );

        let item = builder
            .build_query_as::<ContractItem>()
            .fetch_one(pool)
            .await?;

        if dto.unit_price.is_some() || dto.quantity.is_some() {
            let qty = dto.quantity.unwrap_or(item.quantity);
            let price = dto
                .unit_price
                .map(from_decimal)
                .or(item.unit_price)
                .unwrap_or(0.0);
            let new_total = qty as f64 * price;
            sqlx::query("UPDATE contract_items SET total_price = $1 WHERE id = $2")
                .bind(new_total)
                .bind(item_id)
                .execute(pool)
                .await?;

            return sqlx::query_as::<_, ContractItem>(
                "SELECT id, contract_id, pipe_type, grade, od, wt, quantity, unit_price, \
                 total_price, notes, created_at FROM contract_items WHERE id = $1",
            )
            .bind(item_id)
            .fetch_one(pool)
            .await;
        }

        Ok(item)
    }

    /// Hard DELETE from `contract_items`.
    pub async fn delete_item(pool: &PgPool, item_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM contract_items WHERE id = $1")
            .bind(item_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// INSERT a payment milestone.
    pub async fn create_payment(
        pool: &PgPool,
        contract_id: i64,
        dto: &CreatePaymentRequest,
    ) -> Result<ContractPayment, sqlx::Error> {
        sqlx::query_as::<_, ContractPayment>(
            "INSERT INTO contract_payments (contract_id, due_date, amount, payment_type, notes) \
             VALUES ($1, $2, $3, $4, $5) \
             RETURNING id, contract_id, due_date, amount, payment_type, is_paid, paid_date, \
               notes, created_at",
        )
        .bind(contract_id)
        .bind(&dto.due_date)
        .bind(from_decimal(dto.amount))
        .bind(&dto.payment_type)
        .bind(&dto.notes)
        .fetch_one(pool)
        .await
    }

    /// Dynamic UPDATE of payment fields (due_date, amount, is_paid, etc.).
    /// Only supplied fields change. Returns the updated `ContractPayment`.
    pub async fn update_payment(
        pool: &PgPool,
        payment_id: i64,
        dto: &UpdatePaymentRequest,
    ) -> Result<ContractPayment, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE contract_payments SET");

        let mut sep = false;
        if let Some(ref val) = dto.due_date {
            if sep {
                builder.push(", ");
            }
            builder.push(" due_date = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.amount {
            if sep {
                builder.push(", ");
            }
            builder.push(" amount = ");
            builder.push_bind(from_decimal(val));
            sep = true;
        }
        if let Some(ref val) = dto.payment_type {
            if sep {
                builder.push(", ");
            }
            builder.push(" payment_type = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.is_paid {
            if sep {
                builder.push(", ");
            }
            builder.push(" is_paid = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(ref val) = dto.paid_date {
            if sep {
                builder.push(", ");
            }
            builder.push(" paid_date = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(ref val) = dto.notes {
            if sep {
                builder.push(", ");
            }
            builder.push(" notes = ");
            builder.push_bind(val);
            sep = true;
        }

        if !sep {
            return sqlx::query_as::<_, ContractPayment>(
                "SELECT id, contract_id, due_date, amount, payment_type, is_paid, \
                 paid_date, notes, created_at FROM contract_payments WHERE id = $1",
            )
            .bind(payment_id)
            .fetch_one(pool)
            .await;
        }

        builder.push(" WHERE id = ");
        builder.push_bind(payment_id);
        builder.push(
            " RETURNING id, contract_id, due_date, amount, payment_type, is_paid, \
            paid_date, notes, created_at",
        );

        builder
            .build_query_as::<ContractPayment>()
            .fetch_one(pool)
            .await
    }

    /// SELECT payments by contract, ordered by `due_date`.
    pub async fn find_payments_by_contract(
        pool: &PgPool,
        contract_id: i64,
    ) -> Result<Vec<ContractPayment>, sqlx::Error> {
        sqlx::query_as::<_, ContractPayment>(
            "SELECT id, contract_id, due_date, amount, payment_type, is_paid, paid_date, \
             notes, created_at \
             FROM contract_payments WHERE contract_id = $1 ORDER BY due_date",
        )
        .bind(contract_id)
        .fetch_all(pool)
        .await
    }

    /// SELECT a payment by primary key. Returns `None` if not found.
    pub async fn find_payment_by_id(
        pool: &PgPool,
        payment_id: i64,
    ) -> Result<Option<ContractPayment>, sqlx::Error> {
        sqlx::query_as::<_, ContractPayment>(
            "SELECT id, contract_id, due_date, amount, payment_type, is_paid, paid_date, \
             notes, created_at \
             FROM contract_payments WHERE id = $1",
        )
        .bind(payment_id)
        .fetch_optional(pool)
        .await
    }

    // ━━━ Transaction-safe variants (used inside BEGIN IMMEDIATE) ━━━

    /// [`next_contract_no`] variant that runs on an existing connection (inside a tx).
    async fn next_contract_no_conn(
        executor: &mut SqliteConnection,
        contract_type: &str,
    ) -> Result<String, sqlx::Error> {
        let prefix = match contract_type {
            "sales" => "CT-SAL",
            "purchase" => "CT-PUR",
            _ => "CT",
        };
        let like = format!("{}%", prefix);
        let row: (Option<String>,) =
            sqlx::query_as("SELECT MAX(contract_no) FROM contracts WHERE contract_no LIKE $1")
                .bind(&like)
                .fetch_optional(executor)
                .await?
                .unwrap_or((None,));

        let next_seq = match row.0 {
            Some(last) => {
                let parts: Vec<&str> = last.split('-').collect();
                let num_str = parts.last().unwrap_or(&"000000");
                let num: i64 = num_str.parse().unwrap_or(0);
                num + 1
            }
            None => 1,
        };

        Ok(format!("{}-{:06}", prefix, next_seq))
    }

    /// [`create`] variant that runs on an existing connection (inside a tx).
    pub async fn create_conn(
        executor: &mut SqliteConnection,
        dto: &CreateContractRequest,
    ) -> Result<Contract, sqlx::Error> {
        let contract_no = Self::next_contract_no_conn(&mut *executor, &dto.contract_type).await?;

        sqlx::query_as::<_, Contract>(
            "INSERT INTO contracts (contract_no, contract_type, title, party_a, party_b, \
             sign_date, start_date, end_date, total_amount, status, notes) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, 'draft', $9) \
             RETURNING id, contract_no, contract_type, title, party_a, party_b, \
               sign_date, start_date, end_date, total_amount, status, notes, created_by, \
               created_at, updated_at, deleted_at",
        )
        .bind(&contract_no)
        .bind(&dto.contract_type)
        .bind(&dto.title)
        .bind(&dto.party_a)
        .bind(&dto.party_b)
        .bind(&dto.sign_date)
        .bind(&dto.start_date)
        .bind(&dto.end_date)
        .bind(&dto.notes)
        .fetch_one(&mut *executor)
        .await
    }

    /// [`create_items`] variant that runs on an existing connection (inside a tx).
    pub async fn create_items_conn(
        executor: &mut SqliteConnection,
        contract_id: i64,
        items: &[CreateContractItemRequest],
    ) -> Result<Vec<ContractItem>, sqlx::Error> {
        let mut results = Vec::with_capacity(items.len());
        for item in items {
            let total_price = item
                .unit_price
                .map(|p| from_decimal(p) * item.quantity as f64)
                .unwrap_or(0.0);
            let row = sqlx::query_as::<_, ContractItem>(
                "INSERT INTO contract_items (contract_id, pipe_type, grade, od, wt, \
                 quantity, unit_price, total_price, notes) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
                 RETURNING id, contract_id, pipe_type, grade, od, wt, quantity, \
                   unit_price, total_price, notes, created_at",
            )
            .bind(contract_id)
            .bind(&item.pipe_type)
            .bind(&item.grade)
            .bind(item.od)
            .bind(item.wt)
            .bind(item.quantity)
            .bind(from_decimal_opt(item.unit_price))
            .bind(total_price)
            .bind(&item.notes)
            .fetch_one(&mut *executor)
            .await?;
            results.push(row);
        }
        Ok(results)
    }

    /// Guarded status UPDATE: only succeeds when current status matches `current_status`.
    /// Returns `None` when no row was updated (TOCTOU detected).
    pub async fn update_status_if_current(
        executor: &mut SqliteConnection,
        id: i64,
        current_status: &str,
        new_status: &str,
    ) -> Result<Option<Contract>, sqlx::Error> {
        sqlx::query_as::<_, Contract>(
            "UPDATE contracts SET status = $1, updated_at = NOW() \
             WHERE id = $2 AND status = $3 AND deleted_at IS NULL \
             RETURNING id, contract_no, contract_type, title, party_a, party_b, sign_date, \
               start_date, end_date, total_amount, status, notes, created_by, created_at, \
               updated_at, deleted_at",
        )
        .bind(new_status)
        .bind(id)
        .bind(current_status)
        .fetch_optional(executor)
        .await
    }

    /// [`update_total_amount`] variant that runs on an existing connection (inside a tx).
    pub async fn update_total_amount_conn(
        executor: &mut SqliteConnection,
        id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE contracts SET total_amount = (SELECT COALESCE(SUM(total_price), 0) \
             FROM contract_items WHERE contract_id = $1), updated_at = NOW() \
             WHERE id = $2 AND deleted_at IS NULL",
        )
        .bind(id)
        .bind(id)
        .execute(executor)
        .await?;
        Ok(())
    }

    /// [`update`] variant that runs on an existing connection (inside a tx).
    pub async fn update_conn(
        executor: &mut SqliteConnection,
        id: i64,
        dto: &UpdateContractRequest,
    ) -> Result<Contract, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE contracts SET updated_at = NOW()");

        if let Some(ref val) = dto.title {
            builder.push(", title = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.party_a {
            builder.push(", party_a = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.party_b {
            builder.push(", party_b = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.sign_date {
            builder.push(", sign_date = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.start_date {
            builder.push(", start_date = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.end_date {
            builder.push(", end_date = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.notes {
            builder.push(", notes = ");
            builder.push_bind(val);
        }

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(
            " AND deleted_at IS NULL RETURNING id, contract_no, contract_type, \
            title, party_a, party_b, sign_date, start_date, end_date, total_amount, \
            status, notes, created_by, created_at, updated_at, deleted_at",
        );

        builder.build_query_as::<Contract>().fetch_one(&mut *executor).await
    }

    /// [`find_by_id`] variant that runs on an existing connection (inside a tx).
    pub async fn find_by_id_conn(
        executor: &mut SqliteConnection,
        id: i64,
    ) -> Result<Option<Contract>, sqlx::Error> {
        sqlx::query_as::<_, Contract>(
            "SELECT id, contract_no, contract_type, title, party_a, party_b, sign_date, \
             start_date, end_date, total_amount, status, notes, created_by, created_at, \
             updated_at, deleted_at \
             FROM contracts WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(executor)
        .await
    }

    /// [`update_item`] variant that runs on an existing connection (inside a tx).
    pub async fn update_item_conn(
        executor: &mut SqliteConnection,
        item_id: i64,
        dto: &UpdateContractItemRequest,
    ) -> Result<ContractItem, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE contract_items SET");

        let mut sep = false;
        if let Some(ref val) = dto.pipe_type {
            if sep {
                builder.push(", ");
            }
            builder.push(" pipe_type = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(ref val) = dto.grade {
            if sep {
                builder.push(", ");
            }
            builder.push(" grade = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.od {
            if sep {
                builder.push(", ");
            }
            builder.push(" od = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.wt {
            if sep {
                builder.push(", ");
            }
            builder.push(" wt = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.quantity {
            if sep {
                builder.push(", ");
            }
            builder.push(" quantity = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.unit_price {
            if sep {
                builder.push(", ");
            }
            builder.push(" unit_price = ");
            builder.push_bind(from_decimal(val));
            sep = true;
        }
        if let Some(ref val) = dto.notes {
            if sep {
                builder.push(", ");
            }
            builder.push(" notes = ");
            builder.push_bind(val);
            sep = true;
        }

        if !sep {
            return sqlx::query_as::<_, ContractItem>(
                "SELECT id, contract_id, pipe_type, grade, od, wt, quantity, unit_price, \
                 total_price, notes, created_at FROM contract_items WHERE id = $1",
            )
            .bind(item_id)
            .fetch_one(&mut *executor)
            .await;
        }

        builder.push(" WHERE id = ");
        builder.push_bind(item_id);
        builder.push(
            " RETURNING id, contract_id, pipe_type, grade, od, wt, quantity, \
             unit_price, total_price, notes, created_at",
        );

        let item = builder
            .build_query_as::<ContractItem>()
            .fetch_one(&mut *executor)
            .await?;

        if dto.unit_price.is_some() || dto.quantity.is_some() {
            let qty = dto.quantity.unwrap_or(item.quantity);
            let price = dto
                .unit_price
                .map(from_decimal)
                .or(item.unit_price)
                .unwrap_or(0.0);
            let new_total = qty as f64 * price;
            sqlx::query("UPDATE contract_items SET total_price = $1 WHERE id = $2")
                .bind(new_total)
                .bind(item_id)
                .execute(&mut *executor)
                .await?;

            return sqlx::query_as::<_, ContractItem>(
                "SELECT id, contract_id, pipe_type, grade, od, wt, quantity, unit_price, \
                 total_price, notes, created_at FROM contract_items WHERE id = $1",
            )
            .bind(item_id)
            .fetch_one(&mut *executor)
            .await;
        }

        Ok(item)
    }

    /// [`delete_item`] variant that runs on an existing connection (inside a tx).
    pub async fn delete_item_conn(
        executor: &mut SqliteConnection,
        item_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM contract_items WHERE id = $1")
            .bind(item_id)
            .execute(&mut *executor)
            .await?;
        Ok(())
    }

    /// DELETE all items for a contract whose id is NOT in `keep_ids`.
    /// Used during bulk item replacement on contract update.
    /// When `keep_ids` is empty, deletes ALL items for the contract.
    pub async fn delete_items_not_in_conn(
        executor: &mut SqliteConnection,
        contract_id: i64,
        keep_ids: &[i64],
    ) -> Result<(), sqlx::Error> {
        if keep_ids.is_empty() {
            sqlx::query("DELETE FROM contract_items WHERE contract_id = $1")
                .bind(contract_id)
                .execute(&mut *executor)
                .await?;
        } else {
            let placeholders: Vec<String> =
                keep_ids.iter().enumerate().map(|(i, _)| format!("${}", i + 2)).collect();
            let sql = format!(
                "DELETE FROM contract_items WHERE contract_id = $1 AND id NOT IN ({})",
                placeholders.join(",")
            );
            let mut q = sqlx::query(&sql);
            q = q.bind(contract_id);
            for id in keep_ids {
                q = q.bind(*id);
            }
            q.execute(&mut *executor).await?;
        }
        Ok(())
    }

    /// Hard DELETE from `contract_payments`.
    pub async fn delete_payment(pool: &PgPool, payment_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM contract_payments WHERE id = $1")
            .bind(payment_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
