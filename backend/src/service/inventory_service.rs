use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{
    InboundRecord, InboundItem, OutboundRecord, OutboundItem,
    InventoryCheck, InventoryCheckItem,
};
use crate::error::{AppError, AppResult};
use crate::repository::inventory_repo::{
    InboundRepo, OutboundRepo, StockRepo, InventoryCheckRepo,
};

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateInboundItemDto {
    pub pipe_type: String,
    pub pipe_id: String,
    pub confirmed: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInboundDto {
    pub inbound_type: String,
    pub supplier_id: Option<String>,
    pub order_id: Option<String>,
    pub items: Vec<CreateInboundItemDto>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InboundFilter {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub inbound_type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub supplier_id: Option<String>,
}

impl InboundFilter {
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }
    pub fn page_size(&self) -> i64 {
        self.page_size.unwrap_or(20).clamp(1, 100)
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateOutboundItemDto {
    pub pipe_type: String,
    pub pipe_id: String,
    pub confirmed: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOutboundDto {
    pub outbound_type: String,
    pub customer_id: Option<String>,
    pub order_id: Option<String>,
    pub items: Vec<CreateOutboundItemDto>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OutboundFilter {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub outbound_type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub customer_id: Option<String>,
}

impl OutboundFilter {
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }
    pub fn page_size(&self) -> i64 {
        self.page_size.unwrap_or(20).clamp(1, 100)
    }
}

#[derive(Debug, Serialize)]
pub struct StockSummary {
    pub total_in_stock: i64,
    pub total_seamless: i64,
    pub total_screen: i64,
    pub by_status: HashMap<String, i64>,
}

#[derive(Debug, Serialize)]
pub struct StockByGrade {
    pub grade: String,
    pub seamless: i64,
    pub screen: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct StockByLocation {
    pub location: String,
    pub seamless: i64,
    pub screen: i64,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateInventoryCheckItemDto {
    pub pipe_type: String,
    pub pipe_id: String,
    pub expected: Option<bool>,
    pub confirmed: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInventoryCheckDto {
    pub check_type: String,
    pub items: Vec<CreateInventoryCheckItemDto>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InventoryCheckFilter {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl InventoryCheckFilter {
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }
    pub fn page_size(&self) -> i64 {
        self.page_size.unwrap_or(20).clamp(1, 100)
    }
}

#[derive(Debug, Serialize)]
pub struct InboundDetail {
    pub record: InboundRecord,
    pub items: Vec<InboundItem>,
}

#[derive(Debug, Serialize)]
pub struct OutboundDetail {
    pub record: OutboundRecord,
    pub items: Vec<OutboundItem>,
}

#[derive(Debug, Serialize)]
pub struct InventoryCheckDetail {
    pub check: InventoryCheck,
    pub items: Vec<InventoryCheckItem>,
}

#[derive(Debug, Serialize)]
pub struct InventoryCheckListResponse {
    pub id: String,
    pub check_no: String,
    pub check_type: String,
    pub operator_id: String,
    pub total_expected: i32,
    pub total_confirmed: i32,
    pub total_missing: i32,
    pub notes: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct InventoryService {
    db: SqlitePool,
    inbound_repo: InboundRepo,
    outbound_repo: OutboundRepo,
    stock_repo: StockRepo,
    check_repo: InventoryCheckRepo,
}

impl InventoryService {
    pub fn new(db: SqlitePool) -> Self {
        Self {
            inbound_repo: InboundRepo::new(db.clone()),
            outbound_repo: OutboundRepo::new(db.clone()),
            stock_repo: StockRepo::new(db.clone()),
            check_repo: InventoryCheckRepo::new(db.clone()),
            db,
        }
    }

    fn gen_id() -> String {
        Uuid::new_v4().to_string()
    }

    fn now() -> String {
        Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
    }

    fn today_prefix() -> String {
        Utc::now().format("IN-%Y%m%d").to_string()
    }

    fn today_outbound_prefix() -> String {
        Utc::now().format("OUT-%Y%m%d").to_string()
    }

    fn today_check_prefix() -> String {
        Utc::now().format("CHK-%Y%m%d").to_string()
    }

    async fn next_inbound_no(&self) -> AppResult<String> {
        let prefix = Self::today_prefix();
        let count = self.inbound_repo.count_today_inbound(&prefix).await?;
        Ok(format!("{}-{:03}", prefix, count + 1))
    }

    async fn next_outbound_no(&self) -> AppResult<String> {
        let prefix = Self::today_outbound_prefix();
        let count = self.outbound_repo.count_today_outbound(&prefix).await?;
        Ok(format!("{}-{:03}", prefix, count + 1))
    }

    async fn next_check_no(&self) -> AppResult<String> {
        let prefix = Self::today_check_prefix();
        let count = self.check_repo.count_today_checks(&prefix).await?;
        Ok(format!("{}-{:03}", prefix, count + 1))
    }

    async fn set_pipe_status(
        &self,
        pipe_type: &str,
        pipe_id: &str,
        status: &str,
    ) -> AppResult<()> {
        match pipe_type {
            "seamless" => {
                sqlx::query("UPDATE seamless_pipes SET status = ?1, updated_at = ?2 WHERE id = ?3")
                    .bind(status)
                    .bind(Self::now())
                    .bind(pipe_id)
                    .execute(&self.db)
                    .await?;
            }
            "screen" => {
                sqlx::query("UPDATE screen_pipes SET status = ?1, updated_at = ?2 WHERE id = ?3")
                    .bind(status)
                    .bind(Self::now())
                    .bind(pipe_id)
                    .execute(&self.db)
                    .await?;
            }
            _ => {
                return Err(AppError::BadRequest(format!("Invalid pipe_type: {}", pipe_type)));
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Inbound
    // -----------------------------------------------------------------------

    pub async fn create_inbound(
        &self,
        dto: CreateInboundDto,
        operator_id: &str,
    ) -> AppResult<InboundDetail> {
        if dto.items.is_empty() {
            return Err(AppError::BadRequest("At least one item is required".into()));
        }

        // Validate pipe existence
        for item in &dto.items {
            let exists = self.stock_repo.validate_pipe_exists(&item.pipe_type, &item.pipe_id).await?;
            if !exists {
                return Err(AppError::BadRequest(format!(
                    "Pipe {} ({}) not found",
                    item.pipe_id, item.pipe_type
                )));
            }
            // Check pipe isn't already inbound or outbound (must not already be in_stock)
            let already = self.stock_repo.validate_pipe_in_stock(&item.pipe_type, &item.pipe_id).await?;
            if already {
                return Err(AppError::BadRequest(format!(
                    "Pipe {} ({}) is already in stock", item.pipe_id, item.pipe_type
                )));
            }
        }

        let now = Self::now();
        let record_id = Self::gen_id();
        let inbound_no = self.next_inbound_no().await?;
        let total_items = dto.items.len() as i32;

        let record = InboundRecord {
            id: record_id.clone(),
            inbound_no,
            inbound_type: dto.inbound_type,
            supplier_id: dto.supplier_id,
            order_id: dto.order_id,
            operator_id: operator_id.to_string(),
            total_items,
            notes: dto.notes,
            created_at: now.clone(),
        };

        let mut tx = self.db.begin().await?;

        InboundRepo::create_inbound_tx(&mut tx, &record).await?;

        let mut items = Vec::with_capacity(dto.items.len());
        for it in &dto.items {
            let item = InboundItem {
                id: Self::gen_id(),
                inbound_id: record_id.clone(),
                pipe_type: it.pipe_type.clone(),
                pipe_id: it.pipe_id.clone(),
                confirmed: it.confirmed.unwrap_or(true),
                notes: it.notes.clone(),
            };
            InboundRepo::create_inbound_item_tx(&mut tx, &item).await?;
            items.push(item);
        }

        tx.commit().await?;

        // Update pipe status to in_stock (outside transaction; these are simple)
        for it in &dto.items {
            self.set_pipe_status(&it.pipe_type, &it.pipe_id, "in_stock").await?;
        }

        Ok(InboundDetail { record, items })
    }

    pub async fn list_inbound_records(
        &self,
        filter: InboundFilter,
    ) -> AppResult<(Vec<InboundRecord>, i64)> {
        self.inbound_repo
            .list(
                filter.page(),
                filter.page_size(),
                filter.inbound_type.as_deref(),
                filter.start_date.as_deref(),
                filter.end_date.as_deref(),
                filter.supplier_id.as_deref(),
            )
            .await
    }

    pub async fn get_inbound_detail(&self, id: &str) -> AppResult<InboundDetail> {
        let record = self.inbound_repo.find_by_id(id).await?;
        let items = self.inbound_repo.find_items_by_inbound_id(id).await?;
        Ok(InboundDetail { record, items })
    }

    // -----------------------------------------------------------------------
    // Outbound
    // -----------------------------------------------------------------------

    pub async fn create_outbound(
        &self,
        dto: CreateOutboundDto,
        operator_id: &str,
    ) -> AppResult<OutboundDetail> {
        if dto.items.is_empty() {
            return Err(AppError::BadRequest("At least one item is required".into()));
        }

        // Validate pipes exist and are in_stock
        for item in &dto.items {
            let in_stock = self.stock_repo.validate_pipe_in_stock(&item.pipe_type, &item.pipe_id).await?;
            if !in_stock {
                return Err(AppError::BadRequest(format!(
                    "Pipe {} ({}) is not in stock",
                    item.pipe_id, item.pipe_type
                )));
            }
        }

        let now = Self::now();
        let record_id = Self::gen_id();
        let outbound_no = self.next_outbound_no().await?;
        let total_items = dto.items.len() as i32;

        let record = OutboundRecord {
            id: record_id.clone(),
            outbound_no,
            outbound_type: dto.outbound_type,
            customer_id: dto.customer_id,
            order_id: dto.order_id,
            operator_id: operator_id.to_string(),
            total_items,
            notes: dto.notes,
            created_at: now.clone(),
        };

        let mut tx = self.db.begin().await?;

        OutboundRepo::create_outbound_tx(&mut tx, &record).await?;

        let mut items = Vec::with_capacity(dto.items.len());
        for it in &dto.items {
            let item = OutboundItem {
                id: Self::gen_id(),
                outbound_id: record_id.clone(),
                pipe_type: it.pipe_type.clone(),
                pipe_id: it.pipe_id.clone(),
                confirmed: it.confirmed.unwrap_or(true),
                notes: it.notes.clone(),
            };
            OutboundRepo::create_outbound_item_tx(&mut tx, &item).await?;
            items.push(item);
        }

        tx.commit().await?;

        // Update pipe status to outbound
        for it in &dto.items {
            self.set_pipe_status(&it.pipe_type, &it.pipe_id, "outbound").await?;
        }

        Ok(OutboundDetail { record, items })
    }

    pub async fn list_outbound_records(
        &self,
        filter: OutboundFilter,
    ) -> AppResult<(Vec<OutboundRecord>, i64)> {
        self.outbound_repo
            .list(
                filter.page(),
                filter.page_size(),
                filter.outbound_type.as_deref(),
                filter.start_date.as_deref(),
                filter.end_date.as_deref(),
                filter.customer_id.as_deref(),
            )
            .await
    }

    pub async fn get_outbound_detail(&self, id: &str) -> AppResult<OutboundDetail> {
        let record = self.outbound_repo.find_by_id(id).await?;
        let items = self.outbound_repo.find_items_by_outbound_id(id).await?;
        Ok(OutboundDetail { record, items })
    }

    // -----------------------------------------------------------------------
    // Stock
    // -----------------------------------------------------------------------

    pub async fn get_stock_summary(&self) -> AppResult<StockSummary> {
        let total_in_stock = self.stock_repo.total_in_stock().await?;
        let seamless_total = self.stock_repo.count_seamless_pipes().await?;
        let screen_total = self.stock_repo.count_screen_pipes().await?;

        let mut by_status: HashMap<String, i64> = HashMap::new();

        for (status, count) in self.stock_repo.seamless_by_status().await? {
            *by_status.entry(format!("seamless_{}", status)).or_insert(0) += count;
        }
        for (status, count) in self.stock_repo.screen_by_status().await? {
            *by_status.entry(format!("screen_{}", status)).or_insert(0) += count;
        }

        Ok(StockSummary {
            total_in_stock,
            total_seamless: seamless_total,
            total_screen: screen_total,
            by_status,
        })
    }

    pub async fn get_stock_by_grade(&self) -> AppResult<Vec<StockByGrade>> {
        let rows = self.stock_repo.stock_by_grade().await?;
        Ok(rows
            .into_iter()
            .map(|(grade, seamless, screen)| StockByGrade {
                grade,
                seamless,
                screen,
                total: seamless + screen,
            })
            .collect())
    }

    pub async fn get_stock_by_location(&self) -> AppResult<Vec<StockByLocation>> {
        let rows = self.stock_repo.stock_by_location().await?;
        Ok(rows
            .into_iter()
            .map(|(location, seamless, screen)| StockByLocation {
                location,
                seamless,
                screen,
                total: seamless + screen,
            })
            .collect())
    }

    // -----------------------------------------------------------------------
    // Inventory Checks
    // -----------------------------------------------------------------------

    pub async fn create_inventory_check(
        &self,
        dto: CreateInventoryCheckDto,
        operator_id: &str,
    ) -> AppResult<InventoryCheckDetail> {
        if dto.items.is_empty() {
            return Err(AppError::BadRequest("At least one item is required".into()));
        }

        let now = Self::now();
        let check_id = Self::gen_id();
        let check_no = self.next_check_no().await?;

        let total_expected = dto.items.len() as i32;
        let total_confirmed = dto
            .items
            .iter()
            .filter(|it| it.confirmed.unwrap_or(false))
            .count() as i32;
        let total_missing = total_expected - total_confirmed;
        let status = if total_missing == 0 { "completed" } else { "in_progress" };

        let check = InventoryCheck {
            id: check_id.clone(),
            check_no,
            check_type: dto.check_type,
            operator_id: operator_id.to_string(),
            total_expected,
            total_confirmed,
            total_missing,
            notes: dto.notes.clone(),
            status: status.to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        let mut tx = self.db.begin().await?;

        InventoryCheckRepo::create_check_tx(&mut tx, &check).await?;

        let mut items = Vec::with_capacity(dto.items.len());
        for it in &dto.items {
            let item = InventoryCheckItem {
                id: Self::gen_id(),
                check_id: check_id.clone(),
                pipe_type: it.pipe_type.clone(),
                pipe_id: it.pipe_id.clone(),
                expected: it.expected.unwrap_or(true),
                confirmed: it.confirmed.unwrap_or(false),
                notes: it.notes.clone(),
            };
            InventoryCheckRepo::create_check_item_tx(&mut tx, &item).await?;
            items.push(item);
        }

        tx.commit().await?;

        Ok(InventoryCheckDetail { check, items })
    }

    pub async fn get_inventory_check_detail(&self, id: &str) -> AppResult<InventoryCheckDetail> {
        let check = self.check_repo.find_by_id(id).await?;
        let items = self.check_repo.find_items_by_check_id(id).await?;
        Ok(InventoryCheckDetail { check, items })
    }

    pub async fn list_inventory_checks(
        &self,
        filter: InventoryCheckFilter,
    ) -> AppResult<(Vec<InventoryCheckListResponse>, i64)> {
        let (checks, total) = self
            .check_repo
            .list(
                filter.page(),
                filter.page_size(),
                filter.status.as_deref(),
                filter.start_date.as_deref(),
                filter.end_date.as_deref(),
            )
            .await?;

        let resp: Vec<InventoryCheckListResponse> = checks
            .into_iter()
            .map(|c| InventoryCheckListResponse {
                id: c.id,
                check_no: c.check_no,
                check_type: c.check_type,
                operator_id: c.operator_id,
                total_expected: c.total_expected,
                total_confirmed: c.total_confirmed,
                total_missing: c.total_missing,
                notes: c.notes,
                status: c.status,
                created_at: c.created_at,
                updated_at: c.updated_at,
            })
            .collect();

        Ok((resp, total))
    }
}
