use sqlx::{SqlitePool, Transaction};
use crate::domain::{Contract, ContractItem, ContractPayment};
use crate::error::{AppError, AppResult};

// ── Allowed sort columns for SQL injection prevention ───────────────────────

const ALLOWED_SORT_COLUMNS: &[&str] = &[
    "created_at", "contract_no", "contract_type", "status", "total_amount",
    "sign_date", "effective_date", "expiry_date",
];

fn validate_sort_col(sort_by: &str) -> &'static str {
    ALLOWED_SORT_COLUMNS
        .iter()
        .find(|col| **col == sort_by)
        .copied()
        .unwrap_or("created_at")
}

// ── ContractRepo ────────────────────────────────────────────────────────────

pub struct ContractRepo;

impl ContractRepo {
    pub async fn create(pool: &SqlitePool, contract: &Contract) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO contracts \
             (id, contract_no, contract_type, party_id, total_amount, status, \
              sign_date, effective_date, expiry_date, notes, operator_id, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(&contract.id)
        .bind(&contract.contract_no)
        .bind(&contract.contract_type)
        .bind(&contract.party_id)
        .bind(contract.total_amount)
        .bind(&contract.status)
        .bind(&contract.sign_date)
        .bind(&contract.effective_date)
        .bind(&contract.expiry_date)
        .bind(&contract.notes)
        .bind(&contract.operator_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn create_tx(tx: &mut Transaction<'_, sqlx::Sqlite>, contract: &Contract) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO contracts \
             (id, contract_no, contract_type, party_id, total_amount, status, \
              sign_date, effective_date, expiry_date, notes, operator_id, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(&contract.id)
        .bind(&contract.contract_no)
        .bind(&contract.contract_type)
        .bind(&contract.party_id)
        .bind(contract.total_amount)
        .bind(&contract.status)
        .bind(&contract.sign_date)
        .bind(&contract.effective_date)
        .bind(&contract.expiry_date)
        .bind(&contract.notes)
        .bind(&contract.operator_id)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> AppResult<Contract> {
        let c = sqlx::query_as::<_, Contract>(
            "SELECT id, contract_no, contract_type, party_id, total_amount, status, \
                    sign_date, effective_date, expiry_date, notes, operator_id, \
                    created_at, updated_at, deleted_at \
             FROM contracts WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Contract {} not found", id)))?;
        Ok(c)
    }

    pub async fn update(pool: &SqlitePool, id: &str, c: &Contract) -> AppResult<()> {
        let affected = sqlx::query(
            "UPDATE contracts SET \
             contract_type = ?, party_id = ?, total_amount = ?, \
             sign_date = ?, effective_date = ?, expiry_date = ?, \
             notes = ?, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(&c.contract_type)
        .bind(&c.party_id)
        .bind(c.total_amount)
        .bind(&c.sign_date)
        .bind(&c.effective_date)
        .bind(&c.expiry_date)
        .bind(&c.notes)
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFound(format!("Contract {} not found", id)));
        }
        Ok(())
    }

    pub async fn update_tx(tx: &mut Transaction<'_, sqlx::Sqlite>, id: &str, c: &Contract) -> AppResult<()> {
        let affected = sqlx::query(
            "UPDATE contracts SET \
             contract_type = ?, party_id = ?, total_amount = ?, \
             sign_date = ?, effective_date = ?, expiry_date = ?, \
             notes = ?, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(&c.contract_type)
        .bind(&c.party_id)
        .bind(c.total_amount)
        .bind(&c.sign_date)
        .bind(&c.effective_date)
        .bind(&c.expiry_date)
        .bind(&c.notes)
        .bind(id)
        .execute(&mut **tx)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFound(format!("Contract {} not found", id)));
        }
        Ok(())
    }

    pub async fn update_status(pool: &SqlitePool, id: &str, status: &str) -> AppResult<()> {
        let affected = sqlx::query(
            "UPDATE contracts SET status = ?, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFound(format!("Contract {} not found", id)));
        }
        Ok(())
    }

    pub async fn soft_delete(pool: &SqlitePool, id: &str) -> AppResult<()> {
        let affected = sqlx::query(
            "UPDATE contracts SET deleted_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFound(format!("Contract {} not found", id)));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list(
        pool: &SqlitePool,
        contract_type: Option<&str>,
        status: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
        party_id: Option<&str>,
        page: i64,
        page_size: i64,
        sort_by: &str,
        sort_order: &str,
    ) -> AppResult<Vec<Contract>> {
        let sort_col = validate_sort_col(sort_by);
        let order = if sort_order.eq_ignore_ascii_case("asc") {
            "ASC"
        } else {
            "DESC"
        };
        let offset = (page - 1) * page_size;

        // Build dynamic query with parameterised placeholders
        let sql = format!(
            "SELECT id, contract_no, contract_type, party_id, total_amount, status, \
                    sign_date, effective_date, expiry_date, notes, operator_id, \
                    created_at, updated_at, deleted_at \
             FROM contracts \
             WHERE deleted_at IS NULL \
               AND (?1 IS NULL OR contract_type = ?1) \
               AND (?2 IS NULL OR status = ?2) \
               AND (?3 IS NULL OR created_at >= ?3) \
               AND (?4 IS NULL OR created_at <= ?4) \
               AND (?5 IS NULL OR party_id = ?5) \
             ORDER BY {} {} \
             LIMIT ?6 OFFSET ?7",
            sort_col, order
        );

        let rows = sqlx::query_as::<_, Contract>(&sql)
            .bind(contract_type)
            .bind(status)
            .bind(date_from)
            .bind(date_to)
            .bind(party_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    pub async fn count(
        pool: &SqlitePool,
        contract_type: Option<&str>,
        status: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
        party_id: Option<&str>,
    ) -> AppResult<i64> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM contracts \
             WHERE deleted_at IS NULL \
               AND (?1 IS NULL OR contract_type = ?1) \
               AND (?2 IS NULL OR status = ?2) \
               AND (?3 IS NULL OR created_at >= ?3) \
               AND (?4 IS NULL OR created_at <= ?4) \
               AND (?5 IS NULL OR party_id = ?5)",
        )
        .bind(contract_type)
        .bind(status)
        .bind(date_from)
        .bind(date_to)
        .bind(party_id)
        .fetch_one(pool)
        .await?;
        Ok(count.0)
    }
}

// ── ContractItemRepo ────────────────────────────────────────────────────────

pub struct ContractItemRepo;

impl ContractItemRepo {
    /// Replaces all items for a contract (delete old, insert new) in a transaction.
    pub async fn batch_replace(pool: &SqlitePool, contract_id: &str, items: &[ContractItem]) -> AppResult<()> {
        let mut tx = pool.begin().await?;

        sqlx::query("DELETE FROM contract_items WHERE contract_id = ?")
            .bind(contract_id)
            .execute(&mut *tx)
            .await?;

        for item in items {
            sqlx::query(
                "INSERT INTO contract_items \
                 (id, contract_id, description, spec, quantity, unit_price, amount, delivery_date) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&item.id)
            .bind(&item.contract_id)
            .bind(&item.description)
            .bind(&item.spec)
            .bind(item.quantity)
            .bind(item.unit_price)
            .bind(item.amount)
            .bind(&item.delivery_date)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Replaces all items within an existing parent transaction.
    pub async fn batch_replace_tx(tx: &mut Transaction<'_, sqlx::Sqlite>, contract_id: &str, items: &[ContractItem]) -> AppResult<()> {
        sqlx::query("DELETE FROM contract_items WHERE contract_id = ?")
            .bind(contract_id)
            .execute(&mut **tx)
            .await?;

        for item in items {
            sqlx::query(
                "INSERT INTO contract_items \
                 (id, contract_id, description, spec, quantity, unit_price, amount, delivery_date) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&item.id)
            .bind(&item.contract_id)
            .bind(&item.description)
            .bind(&item.spec)
            .bind(item.quantity)
            .bind(item.unit_price)
            .bind(item.amount)
            .bind(&item.delivery_date)
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }

    pub async fn find_by_contract(pool: &SqlitePool, contract_id: &str) -> AppResult<Vec<ContractItem>> {
        let items = sqlx::query_as::<_, ContractItem>(
            "SELECT id, contract_id, description, spec, quantity, unit_price, amount, delivery_date \
             FROM contract_items WHERE contract_id = ?",
        )
        .bind(contract_id)
        .fetch_all(pool)
        .await?;
        Ok(items)
    }
}

// ── ContractPaymentRepo ─────────────────────────────────────────────────────

pub struct ContractPaymentRepo;

impl ContractPaymentRepo {
    pub async fn create(pool: &SqlitePool, payment: &ContractPayment) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO contract_payments \
             (id, contract_id, stage, amount, due_date, paid, notes) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&payment.id)
        .bind(&payment.contract_id)
        .bind(&payment.stage)
        .bind(payment.amount)
        .bind(&payment.due_date)
        .bind(payment.paid)
        .bind(&payment.notes)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn create_tx(tx: &mut Transaction<'_, sqlx::Sqlite>, payment: &ContractPayment) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO contract_payments \
             (id, contract_id, stage, amount, due_date, paid, notes) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&payment.id)
        .bind(&payment.contract_id)
        .bind(&payment.stage)
        .bind(payment.amount)
        .bind(&payment.due_date)
        .bind(payment.paid)
        .bind(&payment.notes)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn update(pool: &SqlitePool, id: &str, payment: &ContractPayment) -> AppResult<()> {
        let affected = sqlx::query(
            "UPDATE contract_payments SET stage = ?, amount = ?, due_date = ?, paid = ?, notes = ? \
             WHERE id = ?",
        )
        .bind(&payment.stage)
        .bind(payment.amount)
        .bind(&payment.due_date)
        .bind(payment.paid)
        .bind(&payment.notes)
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFound(format!("Contract payment {} not found", id)));
        }
        Ok(())
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> AppResult<()> {
        let affected = sqlx::query("DELETE FROM contract_payments WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFound(format!("Contract payment {} not found", id)));
        }
        Ok(())
    }

    pub async fn find_by_contract(pool: &SqlitePool, contract_id: &str) -> AppResult<Vec<ContractPayment>> {
        let payments = sqlx::query_as::<_, ContractPayment>(
            "SELECT id, contract_id, stage, amount, due_date, paid, notes \
             FROM contract_payments WHERE contract_id = ? \
             ORDER BY due_date ASC",
        )
        .bind(contract_id)
        .fetch_all(pool)
        .await?;
        Ok(payments)
    }

    pub async fn update_paid_status(pool: &SqlitePool, id: &str, paid: bool) -> AppResult<()> {
        let affected = sqlx::query("UPDATE contract_payments SET paid = ? WHERE id = ?")
            .bind(paid)
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFound(format!("Contract payment {} not found", id)));
        }
        Ok(())
    }
}
