use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::domain::money::{from_decimal, from_decimal_opt};
use crate::dto::common::PaginationParams;
use crate::dto::contract_dto::{
    ContractDetailResponse, ContractFilterParams, CreateContractItemRequest, CreateContractRequest,
    CreatePaymentRequest, UpdateContractItemRequest, UpdateContractRequest, UpdatePaymentRequest,
};
use crate::error::AppError;
use crate::models::contract::{Contract, ContractItem, ContractPayment};
use crate::repositories::contract_repo::ContractRepo;

/// Contract service handling the full lifecycle of sales and purchase contracts,
/// including CRUD, line-item management, payment schedules, and status transitions.
/// Status flow: `draft` → `active` → `completed` | `terminated`.
pub struct ContractService;

impl ContractService {
    fn valid_contract_type(contract_type: &str) -> bool {
        matches!(contract_type, "sales" | "purchase")
    }

    fn valid_status(status: &str) -> bool {
        matches!(status, "draft" | "active" | "completed" | "terminated")
    }

    fn valid_payment_type(payment_type: &str) -> bool {
        matches!(payment_type, "deposit" | "milestone" | "final")
    }

    fn allowed_transition(current: &str, target: &str) -> bool {
        matches!(
            (current, target),
            ("draft", "active")
                | ("active", "completed")
                | ("active", "terminated")
                | ("draft", "terminated")
        )
    }

    /// Creates a contract (sales or purchase). Validates the contract type, checks
    /// items aren't empty and quantities are positive. Returns the full contract
    /// with items and payment schedule.
    ///
    /// Uses `BEGIN` to serialise concurrent contract creation — the
    /// contract-number generation (MAX+1) and the insert share one transaction,
    /// preventing duplicate numbers under concurrency.
    ///
    /// # Errors
    /// - `AppError::Validation` — bad type, empty items, or quantity ≤ 0
    pub async fn create_contract(
        pool: &PgPool,
        dto: &CreateContractRequest,
    ) -> Result<ContractDetailResponse, AppError> {
        if !Self::valid_contract_type(&dto.contract_type) {
            return Err(AppError::Validation(format!(
                "Invalid contract type '{}'. Must be 'sales' or 'purchase'",
                dto.contract_type
            )));
        }

        if dto.items.is_empty() {
            return Err(AppError::Validation(
                "Contract must have at least one item".into(),
            ));
        }

        for item in &dto.items {
            if item.quantity <= 0 {
                return Err(AppError::Validation(
                    "Item quantity must be positive".into(),
                ));
            }
        }

        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        let contract = match ContractRepo::create_conn(&mut *conn, dto).await {
            Ok(c) => c,
            Err(e) => {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::from(e));
            }
        };

        let items = match ContractRepo::create_items_conn(&mut *conn, contract.id, &dto.items).await {
            Ok(i) => i,
            Err(e) => {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::from(e));
            }
        };

        if let Err(e) = ContractRepo::update_total_amount_conn(&mut *conn, contract.id).await {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
            return Err(AppError::from(e));
        }

        sqlx::query("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        let contract = ContractRepo::find_by_id(pool, contract.id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to retrieve created contract".into()))?;

        let payments = ContractRepo::find_payments_by_contract(pool, contract.id).await?;

        Ok(ContractDetailResponse {
            contract,
            items,
            payments,
        })
    }

    /// Updates contract header fields and optionally replaces line items.
    /// Only works when the contract is in `draft` status.
    ///
    /// When `dto.items` is `Some`, the existing line items are replaced atomically
    /// within the same transaction:
    /// - Items with an `id` matching an existing row → updated
    /// - Items without an `id` → inserted as new rows
    /// - Existing rows whose `id` is not in the incoming set → deleted
    /// - `total_amount` is recomputed as `SUM(total_price)`
    ///
    /// When `dto.items` is `None`, items are left untouched (backward compatible).
    ///
    /// Uses `BEGIN` to serialise concurrent edits — the scalar update,
    /// item replacement, and `total_amount` recalculation share one transaction.
    ///
    /// # Errors
    /// - `AppError::NotFound` — ID doesn't exist or was deleted
    /// - `AppError::Validation` — current status doesn't allow edits, or quantity ≤ 0
    pub async fn update_contract(
        pool: &PgPool,
        id: i64,
        dto: &UpdateContractRequest,
    ) -> Result<Contract, AppError> {
        let existing = ContractRepo::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Contract id={} not found", id)))?;

        if existing.status != "draft" {
            return Err(AppError::Validation(format!(
                "Cannot modify contract in '{}' status. Only 'draft' contracts can be updated",
                existing.status
            )));
        }

        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        let _ = match ContractRepo::update_conn(&mut *conn, id, dto).await {
            Ok(c) => c,
            Err(e) => {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::from(e));
            }
        };

        if let Some(ref items) = dto.items {
            for item in items {
                if let Some(qty) = item.quantity {
                    if qty <= 0 {
                        sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                        return Err(AppError::Validation(
                            "Item quantity must be positive".into(),
                        ));
                    }
                }
            }

            let keep_ids: Vec<i64> = items.iter().filter_map(|i| i.id).collect();

            if let Err(e) =
                ContractRepo::delete_items_not_in_conn(&mut *conn, id, &keep_ids).await
            {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::from(e));
            }

            for item in items {
                if let Some(item_id) = item.id {
                    if let Err(e) =
                        ContractRepo::update_item_conn(&mut *conn, item_id, item).await
                    {
                        sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                        return Err(AppError::from(e));
                    }
                } else {
                    let pipe_type = item.pipe_type.as_deref().unwrap_or("");
                    let grade = item.grade.as_deref().unwrap_or("");
                    let od = item.od.unwrap_or(0.0);
                    let wt = item.wt.unwrap_or(0.0);
                    let quantity = item.quantity.unwrap_or(0);

                    let total_price = item
                        .unit_price
                        .map(|p| from_decimal(p) * quantity as f64)
                        .unwrap_or(0.0);

                    match sqlx::query(
                        "INSERT INTO contract_items (contract_id, pipe_type, grade, od, wt, \
                         quantity, unit_price, total_price, notes) \
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    )
                    .bind(id)
                    .bind(pipe_type)
                    .bind(grade)
                    .bind(od)
                    .bind(wt)
                    .bind(quantity)
                    .bind(from_decimal_opt(item.unit_price))
                    .bind(total_price)
                    .bind(item.notes.as_deref())
                    .execute(&mut *conn)
                    .await
                    {
                        Err(e) => {
                            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                            return Err(AppError::from(e));
                        }
                        Ok(_) => {}
                    }
                }
            }

            if let Err(e) = ContractRepo::update_total_amount_conn(&mut *conn, id).await {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::from(e));
            }
        }

        sqlx::query("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        ContractRepo::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to retrieve updated contract".into()))
    }

    /// Soft-deletes a contract. Only `draft` contracts can be removed — no wiping
    /// active or completed ones.
    ///
    /// # Errors
    /// - `AppError::NotFound` — ID doesn't exist
    /// - `AppError::Validation` — current status doesn't allow deletion
    pub async fn delete_contract(pool: &PgPool, id: i64) -> Result<(), AppError> {
        let existing = ContractRepo::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Contract id={} not found", id)))?;

        if existing.status != "draft" {
            return Err(AppError::Validation(format!(
                "Cannot delete contract in '{}' status. Only 'draft' contracts can be deleted",
                existing.status
            )));
        }

        ContractRepo::delete(pool, id).await?;
        Ok(())
    }

    /// Fetches the full contract detail including items and payment schedule.
    ///
    /// # Errors
    /// - `AppError::NotFound` — ID doesn't exist or was deleted
    pub async fn get_contract_detail(
        pool: &PgPool,
        id: i64,
    ) -> Result<ContractDetailResponse, AppError> {
        let contract = ContractRepo::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Contract id={} not found", id)))?;

        let items = ContractRepo::find_items_by_contract(pool, id).await?;
        let payments = ContractRepo::find_payments_by_contract(pool, id).await?;

        Ok(ContractDetailResponse {
            contract,
            items,
            payments,
        })
    }

    /// Paginates contracts with filters for type, status, party, etc.
    pub async fn list_contracts(
        pool: &PgPool,
        filter: &ContractFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<Contract>, u64), AppError> {
        ContractRepo::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

    /// Updates contract status. Only valid paths allowed: `draft` → `active` →
    /// `completed` | `terminated`. Activating requires a sign date.
    ///
    /// Uses `BEGIN` with a guarded `WHERE status = <current>` update
    /// to prevent TOCTOU races — concurrent transitions (e.g. `completed` vs
    /// `terminated`) cannot silently overwrite each other.
    ///
    /// # Errors
    /// - `AppError::NotFound` — ID doesn't exist
    /// - `AppError::Validation` — invalid status, illegal transition, or missing sign date
    /// - `AppError::OrderCannotModify` — status changed by another request before commit
    pub async fn update_status(
        pool: &PgPool,
        id: i64,
        new_status: &str,
    ) -> Result<Contract, AppError> {
        if !Self::valid_status(new_status) {
            return Err(AppError::Validation(format!(
                "Invalid status '{}'. Must be 'draft', 'active', 'completed', or 'terminated'",
                new_status
            )));
        }

        let existing = ContractRepo::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Contract id={} not found", id)))?;

        if !Self::allowed_transition(&existing.status, new_status) {
            return Err(AppError::Validation(format!(
                "Cannot transition contract from '{}' to '{}'",
                existing.status, new_status
            )));
        }

        if new_status == "active" && existing.sign_date.is_none() {
            return Err(AppError::Validation(
                "Cannot activate contract without a sign date".into(),
            ));
        }

        // Acquire connection and start IMMEDIATE transaction. The initial read
        // above is a stale-snapshot guard; we re-read inside the tx to confirm
        // the status hasn't changed, then issue a guarded UPDATE.
        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        // Re-read current status inside the transaction
        let current = match ContractRepo::find_by_id_conn(&mut *conn, id).await {
            Ok(Some(c)) => c,
            Ok(None) => {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::NotFound(format!("Contract id={} not found", id)));
            }
            Err(e) => {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::from(e));
            }
        };

        // Re-validate transition — status may have changed since the initial read
        if !Self::allowed_transition(&current.status, new_status) {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
            return Err(AppError::Validation(format!(
                "Cannot transition contract from '{}' to '{}'",
                current.status, new_status
            )));
        }

        if new_status == "active" && current.sign_date.is_none() {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
            return Err(AppError::Validation(
                "Cannot activate contract without a sign date".into(),
            ));
        }

        // Guarded UPDATE — only succeeds if status is still what we re-read
        match ContractRepo::update_status_if_current(&mut *conn, id, &current.status, new_status)
            .await
        {
            Ok(Some(contract)) => {
                sqlx::query("COMMIT")
                    .execute(&mut *conn)
                    .await
                    .map_err(AppError::from)?;
                Ok(contract)
            }
            Ok(None) => {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                Err(AppError::OrderCannotModify(
                    "Contract status changed by another request".into(),
                ))
            }
            Err(e) => {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                Err(AppError::from(e))
            }
        }
    }

    // ━━━ Items ━━━

    /// Adds a line item to a contract. Only works when the contract is in `draft`.
    ///
    /// The item INSERT and `total_amount` recalculation share one transaction
    /// so `total_amount` is never stale after a successful call.
    ///
    /// # Errors
    /// - `AppError::NotFound` — contract doesn't exist
    /// - `AppError::Validation` — quantity ≤ 0 or contract isn't in draft
    pub async fn add_item(
        pool: &PgPool,
        contract_id: i64,
        dto: &CreateContractItemRequest,
    ) -> Result<ContractItem, AppError> {
        if dto.quantity <= 0 {
            return Err(AppError::Validation("Quantity must be positive".into()));
        }

        let existing = ContractRepo::find_by_id(pool, contract_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Contract id={} not found", contract_id)))?;

        if existing.status != "draft" {
            return Err(AppError::Validation(format!(
                "Cannot add items to contract in '{}' status",
                existing.status
            )));
        }

        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        let items = match ContractRepo::create_items_conn(
            &mut *conn,
            contract_id,
            std::slice::from_ref(dto),
        )
        .await
        {
            Ok(i) => i,
            Err(e) => {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::from(e));
            }
        };

        if let Err(e) = ContractRepo::update_total_amount_conn(&mut *conn, contract_id).await {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
            return Err(AppError::from(e));
        }

        sqlx::query("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        items
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("Failed to create item".into()))
    }

    /// Updates a contract line item. Only works in `draft` status; validates the
    /// item actually belongs to this contract.
    ///
    /// The item UPDATE and `total_amount` recalculation share one transaction
    /// so `total_amount` is never stale after a successful call.
    ///
    /// # Errors
    /// - `AppError::NotFound` — contract or item doesn't exist
    /// - `AppError::Validation` — not in draft, item doesn't belong, or quantity ≤ 0
    pub async fn update_item(
        pool: &PgPool,
        contract_id: i64,
        item_id: i64,
        dto: &UpdateContractItemRequest,
    ) -> Result<ContractItem, AppError> {
        let existing = ContractRepo::find_by_id(pool, contract_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Contract id={} not found", contract_id)))?;

        if existing.status != "draft" {
            return Err(AppError::Validation(format!(
                "Cannot modify items in contract with '{}' status",
                existing.status
            )));
        }

        let item = ContractRepo::find_item_by_id(pool, item_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Item id={} not found", item_id)))?;

        if item.contract_id != contract_id {
            return Err(AppError::Validation(
                "Item does not belong to this contract".into(),
            ));
        }

        if let Some(qty) = dto.quantity {
            if qty <= 0 {
                return Err(AppError::Validation("Quantity must be positive".into()));
            }
        }

        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        let result = match ContractRepo::update_item_conn(&mut *conn, item_id, dto).await {
            Ok(r) => r,
            Err(e) => {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::from(e));
            }
        };

        if let Err(e) = ContractRepo::update_total_amount_conn(&mut *conn, contract_id).await {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
            return Err(AppError::from(e));
        }

        sqlx::query("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(result)
    }

    /// Deletes a line item from a contract. Only allowed in `draft` status.
    ///
    /// The item DELETE and `total_amount` recalculation share one transaction
    /// so `total_amount` is never stale after a successful call.
    ///
    /// # Errors
    /// - `AppError::NotFound` — contract or item doesn't exist
    /// - `AppError::Validation` — not in draft or item doesn't belong to this contract
    pub async fn delete_item(
        pool: &PgPool,
        contract_id: i64,
        item_id: i64,
    ) -> Result<(), AppError> {
        let existing = ContractRepo::find_by_id(pool, contract_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Contract id={} not found", contract_id)))?;

        if existing.status != "draft" {
            return Err(AppError::Validation(format!(
                "Cannot delete items from contract with '{}' status",
                existing.status
            )));
        }

        let item = ContractRepo::find_item_by_id(pool, item_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Item id={} not found", item_id)))?;

        if item.contract_id != contract_id {
            return Err(AppError::Validation(
                "Item does not belong to this contract".into(),
            ));
        }

        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        if let Err(e) = ContractRepo::delete_item_conn(&mut *conn, item_id).await {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
            return Err(AppError::from(e));
        }

        if let Err(e) = ContractRepo::update_total_amount_conn(&mut *conn, contract_id).await {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
            return Err(AppError::from(e));
        }

        sqlx::query("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    // ━━━ Payments ━━━

    /// Adds a payment schedule to a contract. No-go on terminated contracts.
    /// Validates payment type (`deposit` / `milestone` / `final`) and positive amount.
    ///
    /// # Errors
    /// - `AppError::NotFound` — contract doesn't exist
    /// - `AppError::Validation` — amount ≤ 0, bad payment type, or contract is terminated
    pub async fn add_payment(
        pool: &PgPool,
        contract_id: i64,
        dto: &CreatePaymentRequest,
    ) -> Result<ContractPayment, AppError> {
        if dto.amount <= Decimal::ZERO {
            return Err(AppError::Validation(
                "Payment amount must be positive".into(),
            ));
        }

        if !Self::valid_payment_type(&dto.payment_type) {
            return Err(AppError::Validation(format!(
                "Invalid payment type '{}'. Must be 'deposit', 'milestone', or 'final'",
                dto.payment_type
            )));
        }

        let existing = ContractRepo::find_by_id(pool, contract_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Contract id={} not found", contract_id)))?;

        if existing.status == "terminated" {
            return Err(AppError::Validation(
                "Cannot add payments to a terminated contract".into(),
            ));
        }

        ContractRepo::create_payment(pool, contract_id, dto)
            .await
            .map_err(AppError::from)
    }

    /// Updates a payment schedule. Can't touch payments on terminated contracts.
    /// Validates the amount is positive and the payment type is legit.
    ///
    /// # Errors
    /// - `AppError::NotFound` — contract or payment doesn't exist
    /// - `AppError::Validation` — amount ≤ 0, bad type, wrong contract, or terminated
    pub async fn update_payment(
        pool: &PgPool,
        contract_id: i64,
        payment_id: i64,
        dto: &UpdatePaymentRequest,
    ) -> Result<ContractPayment, AppError> {
        let existing = ContractRepo::find_by_id(pool, contract_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Contract id={} not found", contract_id)))?;

        if existing.status == "terminated" {
            return Err(AppError::Validation(
                "Cannot modify payments on a terminated contract".into(),
            ));
        }

        let payment = ContractRepo::find_payment_by_id(pool, payment_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Payment id={} not found", payment_id)))?;

        if payment.contract_id != contract_id {
            return Err(AppError::Validation(
                "Payment does not belong to this contract".into(),
            ));
        }

        if let Some(amt) = dto.amount {
            if amt <= Decimal::ZERO {
                return Err(AppError::Validation(
                    "Payment amount must be positive".into(),
                ));
            }
        }

        if let Some(ref pt) = dto.payment_type {
            if !Self::valid_payment_type(pt) {
                return Err(AppError::Validation(format!(
                    "Invalid payment type '{}'",
                    pt
                )));
            }
        }

        ContractRepo::update_payment(pool, payment_id, dto)
            .await
            .map_err(AppError::from)
    }

    /// Deletes a payment schedule. Can't remove payments from terminated contracts.
    ///
    /// # Errors
    /// - `AppError::NotFound` — contract or payment doesn't exist
    /// - `AppError::Validation` — payment doesn't belong to this contract or terminated
    pub async fn delete_payment(
        pool: &PgPool,
        contract_id: i64,
        payment_id: i64,
    ) -> Result<(), AppError> {
        let existing = ContractRepo::find_by_id(pool, contract_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Contract id={} not found", contract_id)))?;

        if existing.status == "terminated" {
            return Err(AppError::Validation(
                "Cannot delete payments from a terminated contract".into(),
            ));
        }

        let payment = ContractRepo::find_payment_by_id(pool, payment_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Payment id={} not found", payment_id)))?;

        if payment.contract_id != contract_id {
            return Err(AppError::Validation(
                "Payment does not belong to this contract".into(),
            ));
        }

        ContractRepo::delete_payment(pool, payment_id).await?;
        Ok(())
    }

    /// Returns all payment schedules for a given contract.
    ///
    /// # Errors
    /// - `AppError::NotFound` — contract doesn't exist
    pub async fn get_payments(
        pool: &PgPool,
        contract_id: i64,
    ) -> Result<Vec<ContractPayment>, AppError> {
        ContractRepo::find_by_id(pool, contract_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Contract id={} not found", contract_id)))?;

        ContractRepo::find_payments_by_contract(pool, contract_id)
            .await
            .map_err(AppError::from)
    }
}
