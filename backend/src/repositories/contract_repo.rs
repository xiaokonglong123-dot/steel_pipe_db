use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::dto::common::PaginationParams;
use crate::dto::contract_dto::{
    ContractFilterParams, CreateContractItemRequest, CreateContractRequest,
    CreatePaymentRequest, UpdateContractItemRequest, UpdateContractRequest,
    UpdatePaymentRequest,
};
use crate::models::contract::{Contract, ContractItem, ContractPayment};

pub struct ContractRepo;

impl ContractRepo {
    async fn next_contract_no(pool: &SqlitePool, contract_type: &str) -> Result<String, sqlx::Error> {
        let prefix = match contract_type {
            "sales" => "CT-SAL",
            "purchase" => "CT-PUR",
            _ => "CT",
        };
        let like = format!("{}%", prefix);
        let row: (Option<String>,) = sqlx::query_as(
            "SELECT MAX(contract_no) FROM contracts WHERE contract_no LIKE ?",
        )
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

    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateContractRequest,
    ) -> Result<Contract, sqlx::Error> {
        let contract_no = Self::next_contract_no(pool, &dto.contract_type).await?;

        sqlx::query_as::<_, Contract>(
            "INSERT INTO contracts (contract_no, contract_type, title, party_a, party_b, \
             sign_date, start_date, end_date, total_amount, status, notes) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, 'draft', ?) \
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

    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateContractRequest,
    ) -> Result<Contract, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "UPDATE contracts SET updated_at = datetime('now')",
        );

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
        builder.push(" AND deleted_at IS NULL RETURNING id, contract_no, contract_type, \
            title, party_a, party_b, sign_date, start_date, end_date, total_amount, \
            status, notes, created_by, created_at, updated_at, deleted_at");

        builder.build_query_as::<Contract>().fetch_one(pool).await
    }

    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<Contract>, sqlx::Error> {
        sqlx::query_as::<_, Contract>(
            "SELECT id, contract_no, contract_type, title, party_a, party_b, sign_date, \
             start_date, end_date, total_amount, status, notes, created_by, created_at, \
             updated_at, deleted_at \
             FROM contracts WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE contracts SET deleted_at = datetime('now'), \
             updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn update_status(
        pool: &SqlitePool,
        id: i64,
        status: &str,
    ) -> Result<Contract, sqlx::Error> {
        sqlx::query_as::<_, Contract>(
            "UPDATE contracts SET status = ?, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL \
             RETURNING id, contract_no, contract_type, title, party_a, party_b, sign_date, \
               start_date, end_date, total_amount, status, notes, created_by, created_at, \
               updated_at, deleted_at",
        )
        .bind(status)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn update_total_amount(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE contracts SET total_amount = (SELECT COALESCE(SUM(total_price), 0) \
             FROM contract_items WHERE contract_id = ?), updated_at = datetime('now') \
             WHERE id = ?",
        )
        .bind(id)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn list(
        pool: &SqlitePool,
        filter: &ContractFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<Contract>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        let mut conditions: Vec<String> = vec!["c.deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push("(c.contract_no LIKE ? OR c.title LIKE ? \
                 OR c.party_a LIKE ? OR c.party_b LIKE ?)".into());
                let pattern = format!("%{}%", q);
                bind_values.push(pattern.clone());
                bind_values.push(pattern.clone());
                bind_values.push(pattern.clone());
                bind_values.push(pattern);
            }
        }
        if let Some(ref contract_type) = filter.contract_type {
            conditions.push("c.contract_type = ?".into());
            bind_values.push(contract_type.clone());
        }
        if let Some(ref status) = filter.status {
            conditions.push("c.status = ?".into());
            bind_values.push(status.clone());
        }
        if let Some(ref date_from) = filter.date_from {
            conditions.push("c.sign_date >= ?".into());
            bind_values.push(date_from.clone());
        }
        if let Some(ref date_to) = filter.date_to {
            conditions.push("c.sign_date <= ?".into());
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
             ORDER BY {} {} LIMIT ? OFFSET ?",
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

    // ━━━ Items ━━━

    pub async fn create_items(
        pool: &SqlitePool,
        contract_id: i64,
        items: &[CreateContractItemRequest],
    ) -> Result<Vec<ContractItem>, sqlx::Error> {
        let mut results = Vec::with_capacity(items.len());
        for item in items {
            let total_price = item
                .unit_price
                .map(|p| p * item.quantity as f64)
                .unwrap_or(0.0);
            let row = sqlx::query_as::<_, ContractItem>(
                "INSERT INTO contract_items (contract_id, pipe_type, grade, od, wt, \
                 quantity, unit_price, total_price, notes) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) \
                 RETURNING id, contract_id, pipe_type, grade, od, wt, quantity, \
                   unit_price, total_price, notes, created_at",
            )
            .bind(contract_id)
            .bind(&item.pipe_type)
            .bind(&item.grade)
            .bind(item.od)
            .bind(item.wt)
            .bind(item.quantity)
            .bind(item.unit_price)
            .bind(total_price)
            .bind(&item.notes)
            .fetch_one(pool)
            .await?;
            results.push(row);
        }
        Ok(results)
    }

    pub async fn find_items_by_contract(
        pool: &SqlitePool,
        contract_id: i64,
    ) -> Result<Vec<ContractItem>, sqlx::Error> {
        sqlx::query_as::<_, ContractItem>(
            "SELECT id, contract_id, pipe_type, grade, od, wt, quantity, unit_price, \
             total_price, notes, created_at \
             FROM contract_items WHERE contract_id = ? ORDER BY id",
        )
        .bind(contract_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_item_by_id(
        pool: &SqlitePool,
        item_id: i64,
    ) -> Result<Option<ContractItem>, sqlx::Error> {
        sqlx::query_as::<_, ContractItem>(
            "SELECT id, contract_id, pipe_type, grade, od, wt, quantity, unit_price, \
             total_price, notes, created_at \
             FROM contract_items WHERE id = ?",
        )
        .bind(item_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_item(
        pool: &SqlitePool,
        item_id: i64,
        dto: &UpdateContractItemRequest,
    ) -> Result<ContractItem, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "UPDATE contract_items SET",
        );

        let mut sep = false;
        if let Some(ref val) = dto.pipe_type {
            if sep { builder.push(", "); }
            builder.push(" pipe_type = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(ref val) = dto.grade {
            if sep { builder.push(", "); }
            builder.push(" grade = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.od {
            if sep { builder.push(", "); }
            builder.push(" od = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.wt {
            if sep { builder.push(", "); }
            builder.push(" wt = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.quantity {
            if sep { builder.push(", "); }
            builder.push(" quantity = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.unit_price {
            if sep { builder.push(", "); }
            builder.push(" unit_price = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(ref val) = dto.notes {
            if sep { builder.push(", "); }
            builder.push(" notes = ");
            builder.push_bind(val);
            sep = true;
        }

        if !sep {
            return sqlx::query_as::<_, ContractItem>(
                "SELECT id, contract_id, pipe_type, grade, od, wt, quantity, unit_price, \
                 total_price, notes, created_at FROM contract_items WHERE id = ?",
            )
            .bind(item_id)
            .fetch_one(pool)
            .await;
        }

        builder.push(" WHERE id = ");
        builder.push_bind(item_id);
        builder.push(" RETURNING id, contract_id, pipe_type, grade, od, wt, quantity, \
            unit_price, total_price, notes, created_at");

        let item = builder.build_query_as::<ContractItem>().fetch_one(pool).await?;

        if dto.unit_price.is_some() || dto.quantity.is_some() {
            let qty = dto.quantity.unwrap_or(item.quantity);
            let price = dto.unit_price.or(item.unit_price).unwrap_or(0.0);
            let new_total = qty as f64 * price;
            sqlx::query("UPDATE contract_items SET total_price = ? WHERE id = ?")
                .bind(new_total)
                .bind(item_id)
                .execute(pool)
                .await?;

            return sqlx::query_as::<_, ContractItem>(
                "SELECT id, contract_id, pipe_type, grade, od, wt, quantity, unit_price, \
                 total_price, notes, created_at FROM contract_items WHERE id = ?",
            )
            .bind(item_id)
            .fetch_one(pool)
            .await;
        }

        Ok(item)
    }

    pub async fn delete_item(pool: &SqlitePool, item_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM contract_items WHERE id = ?")
            .bind(item_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    // ━━━ Payments ━━━

    pub async fn create_payment(
        pool: &SqlitePool,
        contract_id: i64,
        dto: &CreatePaymentRequest,
    ) -> Result<ContractPayment, sqlx::Error> {
        sqlx::query_as::<_, ContractPayment>(
            "INSERT INTO contract_payments (contract_id, due_date, amount, payment_type, notes) \
             VALUES (?, ?, ?, ?, ?) \
             RETURNING id, contract_id, due_date, amount, payment_type, is_paid, paid_date, \
               notes, created_at",
        )
        .bind(contract_id)
        .bind(&dto.due_date)
        .bind(dto.amount)
        .bind(&dto.payment_type)
        .bind(&dto.notes)
        .fetch_one(pool)
        .await
    }

    pub async fn update_payment(
        pool: &SqlitePool,
        payment_id: i64,
        dto: &UpdatePaymentRequest,
    ) -> Result<ContractPayment, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "UPDATE contract_payments SET",
        );

        let mut sep = false;
        if let Some(ref val) = dto.due_date {
            if sep { builder.push(", "); }
            builder.push(" due_date = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.amount {
            if sep { builder.push(", "); }
            builder.push(" amount = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(ref val) = dto.payment_type {
            if sep { builder.push(", "); }
            builder.push(" payment_type = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(val) = dto.is_paid {
            if sep { builder.push(", "); }
            builder.push(" is_paid = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(ref val) = dto.paid_date {
            if sep { builder.push(", "); }
            builder.push(" paid_date = ");
            builder.push_bind(val);
            sep = true;
        }
        if let Some(ref val) = dto.notes {
            if sep { builder.push(", "); }
            builder.push(" notes = ");
            builder.push_bind(val);
            sep = true;
        }

        if !sep {
            return sqlx::query_as::<_, ContractPayment>(
                "SELECT id, contract_id, due_date, amount, payment_type, is_paid, \
                 paid_date, notes, created_at FROM contract_payments WHERE id = ?",
            )
            .bind(payment_id)
            .fetch_one(pool)
            .await;
        }

        builder.push(" WHERE id = ");
        builder.push_bind(payment_id);
        builder.push(" RETURNING id, contract_id, due_date, amount, payment_type, is_paid, \
            paid_date, notes, created_at");

        builder.build_query_as::<ContractPayment>().fetch_one(pool).await
    }

    pub async fn find_payments_by_contract(
        pool: &SqlitePool,
        contract_id: i64,
    ) -> Result<Vec<ContractPayment>, sqlx::Error> {
        sqlx::query_as::<_, ContractPayment>(
            "SELECT id, contract_id, due_date, amount, payment_type, is_paid, paid_date, \
             notes, created_at \
             FROM contract_payments WHERE contract_id = ? ORDER BY due_date",
        )
        .bind(contract_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_payment_by_id(
        pool: &SqlitePool,
        payment_id: i64,
    ) -> Result<Option<ContractPayment>, sqlx::Error> {
        sqlx::query_as::<_, ContractPayment>(
            "SELECT id, contract_id, due_date, amount, payment_type, is_paid, paid_date, \
             notes, created_at \
             FROM contract_payments WHERE id = ?",
        )
        .bind(payment_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete_payment(pool: &SqlitePool, payment_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM contract_payments WHERE id = ?")
            .bind(payment_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
