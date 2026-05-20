use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::{Contract, ContractItem, ContractPayment};
use crate::error::{AppError, AppResult};
use crate::repository::contract_repo::{ContractItemRepo, ContractPaymentRepo, ContractRepo};
use crate::AppState;

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ContractItemDto {
    pub description: String,
    pub spec: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
    pub delivery_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContractPaymentDto {
    pub stage: String,
    pub amount: f64,
    pub due_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateContractDto {
    pub contract_type: String,
    pub party_id: String,
    pub sign_date: Option<String>,
    pub effective_date: Option<String>,
    pub expiry_date: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<ContractItemDto>,
    pub payments: Vec<ContractPaymentDto>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContractDto {
    pub contract_type: String,
    pub party_id: String,
    pub total_amount: f64,
    pub sign_date: Option<String>,
    pub effective_date: Option<String>,
    pub expiry_date: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<ContractItemDto>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContractStatusDto {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ContractListFilter {
    pub contract_type: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub party_id: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
}

fn default_page() -> i64 {
    1
}
fn default_page_size() -> i64 {
    20
}
fn default_sort_by() -> String {
    "created_at".to_string()
}
fn default_sort_order() -> String {
    "desc".to_string()
}

#[derive(Debug, Deserialize)]
pub struct AddPaymentDto {
    pub stage: String,
    pub amount: f64,
    pub due_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePaymentDto {
    pub stage: String,
    pub amount: f64,
    pub due_date: Option<String>,
    pub paid: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ContractDetail {
    #[serde(flatten)]
    pub contract: Contract,
    pub items: Vec<ContractItem>,
    pub payments: Vec<ContractPayment>,
    pub party_name: Option<String>,
}

// ── Contract number generator ───────────────────────────────────────────────

async fn generate_contract_no(pool: &SqlitePool) -> AppResult<String> {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let pattern = format!("CT-{}%", today);

    let last: Option<(String,)> = sqlx::query_as(
        "SELECT contract_no FROM contracts WHERE contract_no LIKE ? ORDER BY contract_no DESC LIMIT 1",
    )
    .bind(&pattern)
    .fetch_optional(pool)
    .await?;

    let seq = match last {
        Some((no,)) => {
            let parts: Vec<&str> = no.split('-').collect();
            let last_seq: i32 = parts
                .last()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            last_seq + 1
        }
        None => 1,
    };

    Ok(format!("CT-{}-{:03}", today, seq))
}

/// Fetches the party name given a contract type and party_id.
/// For sales contracts, party_id is a customer; for purchase, it's a supplier.
async fn fetch_party_name(pool: &SqlitePool, contract_type: &str, party_id: &str) -> Option<String> {
    match contract_type {
        "sales" => {
            let row: Option<(String,)> =
                sqlx::query_as("SELECT name FROM customers WHERE id = ?")
                    .bind(party_id)
                    .fetch_optional(pool)
                    .await
                    .ok()?;
            row.map(|r| r.0)
        }
        "purchase" => {
            let row: Option<(String,)> =
                sqlx::query_as("SELECT name FROM suppliers WHERE id = ?")
                    .bind(party_id)
                    .fetch_optional(pool)
                    .await
                    .ok()?;
            row.map(|r| r.0)
        }
        _ => None,
    }
}

// ── ContractService ─────────────────────────────────────────────────────────

pub struct ContractService {
    pool: SqlitePool,
}

impl ContractService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn from_state(state: &Arc<AppState>) -> Self {
        Self::new(state.db.clone())
    }

    // ── Create ──────────────────────────────────────────────────────────────

    pub async fn create(
        &self,
        dto: CreateContractDto,
        operator_id: &str,
    ) -> AppResult<ContractDetail> {
        let party_name = fetch_party_name(&self.pool, &dto.contract_type, &dto.party_id).await;
        if party_name.is_none() {
            return Err(AppError::BadRequest(format!(
                "Party '{}' not found for contract type '{}'",
                dto.party_id, dto.contract_type
            )));
        }

        let contract_id = Uuid::new_v4().to_string();
        let contract_no = generate_contract_no(&self.pool).await?;

        let mut total_amount = 0.0_f64;
        let items: Vec<ContractItem> = dto
            .items
            .iter()
            .map(|item| {
                let amount = item.quantity as f64 * item.unit_price;
                total_amount += amount;
                ContractItem {
                    id: Uuid::new_v4().to_string(),
                    contract_id: contract_id.clone(),
                    description: item.description.clone(),
                    spec: item.spec.clone(),
                    quantity: item.quantity,
                    unit_price: item.unit_price,
                    amount,
                    delivery_date: item.delivery_date.clone(),
                }
            })
            .collect();

        let payments: Vec<ContractPayment> = dto
            .payments
            .iter()
            .map(|p| ContractPayment {
                id: Uuid::new_v4().to_string(),
                contract_id: contract_id.clone(),
                stage: p.stage.clone(),
                amount: p.amount,
                due_date: p.due_date.clone(),
                paid: false,
                notes: p.notes.clone(),
            })
            .collect();

        let contract = Contract {
            id: contract_id.clone(),
            contract_no,
            contract_type: dto.contract_type.clone(),
            party_id: dto.party_id.clone(),
            total_amount,
            status: "draft".to_string(),
            sign_date: dto.sign_date,
            effective_date: dto.effective_date,
            expiry_date: dto.expiry_date,
            notes: dto.notes,
            operator_id: operator_id.to_string(),
            created_at: String::new(),
            updated_at: String::new(),
            deleted_at: None,
        };

        // Persist in transaction
        let mut tx = self.pool.begin().await?;

        ContractRepo::create_tx(&mut tx, &contract).await?;
        ContractItemRepo::batch_replace_tx(&mut tx, &contract_id, &items).await?;

        for payment in &payments {
            ContractPaymentRepo::create_tx(&mut tx, payment).await?;
        }

        tx.commit().await?;

        // Re-fetch for server-populated timestamps
        let saved = ContractRepo::find_by_id(&self.pool, &contract_id).await?;
        let saved_items = ContractItemRepo::find_by_contract(&self.pool, &contract_id).await?;
        let saved_payments = ContractPaymentRepo::find_by_contract(&self.pool, &contract_id).await?;

        Ok(ContractDetail {
            contract: saved,
            items: saved_items,
            payments: saved_payments,
            party_name,
        })
    }

    // ── Update ──────────────────────────────────────────────────────────────

    pub async fn update(
        &self,
        id: &str,
        dto: UpdateContractDto,
    ) -> AppResult<ContractDetail> {
        let existing = ContractRepo::find_by_id(&self.pool, id).await?;

        if existing.status != "draft" {
            return Err(AppError::BadRequest(
                "Only draft contracts can be edited".into(),
            ));
        }

        // Build updated items
        let items: Vec<ContractItem> = dto
            .items
            .iter()
            .map(|item| {
                let amount = item.quantity as f64 * item.unit_price;
                ContractItem {
                    id: Uuid::new_v4().to_string(),
                    contract_id: id.to_string(),
                    description: item.description.clone(),
                    spec: item.spec.clone(),
                    quantity: item.quantity,
                    unit_price: item.unit_price,
                    amount,
                    delivery_date: item.delivery_date.clone(),
                }
            })
            .collect();

        let updated = Contract {
            id: existing.id,
            contract_no: existing.contract_no,
            contract_type: dto.contract_type,
            party_id: dto.party_id,
            total_amount: dto.total_amount,
            status: existing.status,
            sign_date: dto.sign_date,
            effective_date: dto.effective_date,
            expiry_date: dto.expiry_date,
            notes: dto.notes,
            operator_id: existing.operator_id,
            created_at: existing.created_at,
            updated_at: String::new(),
            deleted_at: None,
        };

        let mut tx = self.pool.begin().await?;
        ContractRepo::update_tx(&mut tx, id, &updated).await?;
        ContractItemRepo::batch_replace_tx(&mut tx, id, &items).await?;
        tx.commit().await?;

        let contract = ContractRepo::find_by_id(&self.pool, id).await?;
        let saved_items = ContractItemRepo::find_by_contract(&self.pool, id).await?;
        let payments = ContractPaymentRepo::find_by_contract(&self.pool, id).await?;
        let party_name = fetch_party_name(&self.pool, &contract.contract_type, &contract.party_id).await;

        Ok(ContractDetail {
            contract,
            items: saved_items,
            payments,
            party_name,
        })
    }

    // ── Status workflow ─────────────────────────────────────────────────────

    pub async fn update_status(&self, id: &str, new_status: &str) -> AppResult<Contract> {
        let contract = ContractRepo::find_by_id(&self.pool, id).await?;
        let current = contract.status.as_str();

        let allowed = matches!((current, new_status), ("draft", "active") | ("active", "completed") | ("active", "terminated") | ("draft", "terminated"));

        if !allowed {
            return Err(AppError::BadRequest(format!(
                "Cannot transition contract from '{}' to '{}'",
                current, new_status
            )));
        }

        ContractRepo::update_status(&self.pool, id, new_status).await?;
        ContractRepo::find_by_id(&self.pool, id).await
    }

    // ── Get with items and payments ────────────────────────────────────────

    pub async fn get_with_items(&self, id: &str) -> AppResult<ContractDetail> {
        let contract = ContractRepo::find_by_id(&self.pool, id).await?;
        let items = ContractItemRepo::find_by_contract(&self.pool, id).await?;
        let payments = ContractPaymentRepo::find_by_contract(&self.pool, id).await?;
        let party_name =
            fetch_party_name(&self.pool, &contract.contract_type, &contract.party_id).await;

        Ok(ContractDetail {
            contract,
            items,
            payments,
            party_name,
        })
    }

    // ── List ────────────────────────────────────────────────────────────────

    pub async fn list(
        &self,
        filter: &ContractListFilter,
    ) -> AppResult<(Vec<Contract>, i64)> {
        let page = filter.page.max(1);
        let page_size = filter.page_size.clamp(1, 200);

        let total = ContractRepo::count(
            &self.pool,
            filter.contract_type.as_deref(),
            filter.status.as_deref(),
            filter.date_from.as_deref(),
            filter.date_to.as_deref(),
            filter.party_id.as_deref(),
        )
        .await?;

        let contracts = ContractRepo::list(
            &self.pool,
            filter.contract_type.as_deref(),
            filter.status.as_deref(),
            filter.date_from.as_deref(),
            filter.date_to.as_deref(),
            filter.party_id.as_deref(),
            page,
            page_size,
            &filter.sort_by,
            &filter.sort_order,
        )
        .await?;

        Ok((contracts, total))
    }

    // ── Delete ──────────────────────────────────────────────────────────────

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        let contract = ContractRepo::find_by_id(&self.pool, id).await?;
        if contract.status != "draft" && contract.status != "terminated" {
            return Err(AppError::BadRequest(
                "Only draft or terminated contracts can be deleted".into(),
            ));
        }
        ContractRepo::soft_delete(&self.pool, id).await
    }

    // ── Payments ────────────────────────────────────────────────────────────

    pub async fn add_payment(&self, contract_id: &str, dto: AddPaymentDto) -> AppResult<ContractPayment> {
        ContractRepo::find_by_id(&self.pool, contract_id).await?;

        let payment = ContractPayment {
            id: Uuid::new_v4().to_string(),
            contract_id: contract_id.to_string(),
            stage: dto.stage,
            amount: dto.amount,
            due_date: dto.due_date,
            paid: false,
            notes: dto.notes,
        };

        ContractPaymentRepo::create(&self.pool, &payment).await?;
        Ok(payment)
    }

    pub async fn update_payment(
        &self,
        payment_id: &str,
        dto: UpdatePaymentDto,
    ) -> AppResult<ContractPayment> {
        // Verify payment exists (already checked by repo update)
        let existing = sqlx::query_as::<_, ContractPayment>(
            "SELECT id, contract_id, stage, amount, due_date, paid, notes \
             FROM contract_payments WHERE id = ?",
        )
        .bind(payment_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Payment {} not found", payment_id)))?;

        let updated = ContractPayment {
            id: existing.id,
            contract_id: existing.contract_id,
            stage: dto.stage,
            amount: dto.amount,
            due_date: dto.due_date,
            paid: dto.paid,
            notes: dto.notes,
        };

        ContractPaymentRepo::update(&self.pool, payment_id, &updated).await?;
        Ok(updated)
    }

    pub async fn delete_payment(&self, payment_id: &str) -> AppResult<()> {
        // Also verify the containing contract is draft
        let existing = sqlx::query_as::<_, ContractPayment>(
            "SELECT id, contract_id, stage, amount, due_date, paid, notes \
             FROM contract_payments WHERE id = ?",
        )
        .bind(payment_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Payment {} not found", payment_id)))?;

        let contract = ContractRepo::find_by_id(&self.pool, &existing.contract_id).await?;
        if contract.status != "draft" {
            return Err(AppError::BadRequest(
                "Can only delete payments from draft contracts".into(),
            ));
        }

        ContractPaymentRepo::delete(&self.pool, payment_id).await
    }
}
