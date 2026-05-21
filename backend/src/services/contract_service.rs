use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::contract_dto::{
    ContractDetailResponse, ContractFilterParams, CreateContractItemRequest,
    CreateContractRequest, CreatePaymentRequest, UpdateContractItemRequest,
    UpdateContractRequest, UpdatePaymentRequest,
};
use crate::error::AppError;
use crate::models::contract::{Contract, ContractItem, ContractPayment};
use crate::repositories::contract_repo::ContractRepo;

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

    pub async fn create_contract(
        pool: &SqlitePool,
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

        let contract = ContractRepo::create(pool, dto).await?;
        let items = ContractRepo::create_items(pool, contract.id, &dto.items).await?;
        ContractRepo::update_total_amount(pool, contract.id).await?;

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

    pub async fn update_contract(
        pool: &SqlitePool,
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

        ContractRepo::update(pool, id, dto)
            .await
            .map_err(AppError::from)
    }

    pub async fn delete_contract(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(), AppError> {
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

    pub async fn get_contract_detail(
        pool: &SqlitePool,
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

    pub async fn list_contracts(
        pool: &SqlitePool,
        filter: &ContractFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<Contract>, u64), AppError> {
        ContractRepo::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

    pub async fn update_status(
        pool: &SqlitePool,
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

        ContractRepo::update_status(pool, id, new_status)
            .await
            .map_err(AppError::from)
    }

    // ━━━ Items ━━━

    pub async fn add_item(
        pool: &SqlitePool,
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

        let items = ContractRepo::create_items(pool, contract_id, &[dto.clone()]).await?;
        ContractRepo::update_total_amount(pool, contract_id).await?;

        items.into_iter().next().ok_or_else(|| AppError::Internal("Failed to create item".into()))
    }

    pub async fn update_item(
        pool: &SqlitePool,
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

        let result = ContractRepo::update_item(pool, item_id, dto).await?;
        ContractRepo::update_total_amount(pool, contract_id).await?;

        Ok(result)
    }

    pub async fn delete_item(
        pool: &SqlitePool,
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

        ContractRepo::delete_item(pool, item_id).await?;
        ContractRepo::update_total_amount(pool, contract_id).await?;

        Ok(())
    }

    // ━━━ Payments ━━━

    pub async fn add_payment(
        pool: &SqlitePool,
        contract_id: i64,
        dto: &CreatePaymentRequest,
    ) -> Result<ContractPayment, AppError> {
        if dto.amount <= 0.0 {
            return Err(AppError::Validation("Payment amount must be positive".into()));
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

    pub async fn update_payment(
        pool: &SqlitePool,
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
            if amt <= 0.0 {
                return Err(AppError::Validation("Payment amount must be positive".into()));
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

    pub async fn delete_payment(
        pool: &SqlitePool,
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

    pub async fn get_payments(
        pool: &SqlitePool,
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
