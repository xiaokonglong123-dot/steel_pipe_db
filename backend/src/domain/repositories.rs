use crate::domain::entities::seamless_pipe::{SeamlessPipe, CreateSeamlessPipeCommand};
use crate::domain::value_objects::{PipeNumber, HeatNumber};
use crate::dto::common::{PaginationParams, PipeFilterParams};
use async_trait::async_trait;

/// Pipe repository trait - defines the contract for pipe persistence
#[async_trait]
pub trait PipeRepository: Send + Sync {
    async fn save(&self, pipe: &SeamlessPipe) -> Result<(), crate::error::AppError>;
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<SeamlessPipe>, crate::error::AppError>;
    async fn find_by_pipe_number(&self, number: &PipeNumber) -> Result<Option<SeamlessPipe>, crate::error::AppError>;
    async fn find_by_heat_number(&self, heat_number: &HeatNumber) -> Result<Vec<SeamlessPipe>, crate::error::AppError>;
    async fn list(&self, filter: &PipeFilterParams, pagination: &PaginationParams) -> Result<(Vec<SeamlessPipe>, i64), crate::error::AppError>;
    async fn check_unique(&self, number: &PipeNumber) -> Result<(), crate::error::AppError>;
    async fn delete(&self, id: uuid::Uuid) -> Result<(), crate::error::AppError>;
    async fn update_status(&self, id: uuid::Uuid, status: crate::domain::entities::seamless_pipe::PipeStatus) -> Result<(), crate::error::AppError>;
    async fn update_location(&self, id: uuid::Uuid, location_id: Option<uuid::Uuid>) -> Result<(), crate::error::AppError>;
}

/// Inventory repository trait
#[async_trait]
pub trait InventoryRepository: Send + Sync {
    async fn get_atp(&self, pipe_type: &str, grade: &str, location_id: Option<uuid::Uuid>) -> Result<Vec<(String, String, i64, Option<uuid::Uuid>)>, crate::error::AppError>;
    async fn update_pipe_status(&self, pipe_type: &str, pipe_id: i64, status: &str) -> Result<(), crate::error::AppError>;
    async fn update_pipe_location(&self, pipe_type: &str, pipe_id: i64, location_id: uuid::Uuid) -> Result<(), crate::error::AppError>;
}

/// Location repository trait
#[async_trait]
pub trait LocationRepository: Send + Sync {
    async fn save(&self, location: &crate::models::inventory::Location) -> Result<(), crate::error::AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<crate::models::inventory::Location>, crate::error::AppError>;
    async fn find_by_full_code(&self, code: &str) -> Result<Option<crate::models::inventory::Location>, crate::error::AppError>;
    async fn list(&self, params: &PaginationParams, active_only: bool) -> Result<(Vec<crate::models::inventory::Location>, u64), crate::error::AppError>;
    async fn delete(&self, id: i64) -> Result<(), crate::error::AppError>;
    async fn refresh_used_count(&self, location_id: i64) -> Result<(), crate::error::AppError>;
}

/// Inbound repository trait
#[async_trait]
pub trait InboundRepository: Send + Sync {
    async fn save(&self, record: &crate::models::inventory::InboundRecord, items: &[crate::dto::inventory_dto::InboundPipeItem]) -> Result<(), crate::error::AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<crate::models::inventory::InboundRecord>, crate::error::AppError>;
    async fn find_items(&self, inbound_id: i64) -> Result<Vec<crate::models::inventory::InboundItem>, crate::error::AppError>;
    async fn list(&self, filter: &crate::dto::inventory_dto::InboundFilter) -> Result<(Vec<crate::models::inventory::InboundRecord>, u64), crate::error::AppError>;
    async fn update_status(&self, id: i64, status: &str, reason: Option<&str>) -> Result<(), crate::error::AppError>;
}

/// Outbound repository trait
#[async_trait]
pub trait OutboundRepository: Send + Sync {
    async fn save(&self, record: &crate::models::inventory::OutboundRecord, items: &[crate::dto::inventory_dto::OutboundPipeItem]) -> Result<(), crate::error::AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<crate::models::inventory::OutboundRecord>, crate::error::AppError>;
    async fn find_items(&self, outbound_id: i64) -> Result<Vec<crate::models::inventory::OutboundItem>, crate::error::AppError>;
    async fn list(&self, filter: &crate::dto::inventory_dto::OutboundFilter) -> Result<(Vec<crate::models::inventory::OutboundRecord>, u64), crate::error::AppError>;
    async fn update_status(&self, id: i64, status: &str, reason: Option<&str>) -> Result<(), crate::error::AppError>;
}

/// Purchase order repository trait
#[async_trait]
pub trait PurchaseOrderRepository: Send + Sync {
    async fn save(&self, order: &crate::models::purchase_order::PurchaseOrder, items: &[crate::dto::purchase_dto::PurchaseOrderItemRequest]) -> Result<(), crate::error::AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<crate::models::purchase_order::PurchaseOrder>, crate::error::AppError>;
    async fn find_by_order_no(&self, order_no: &str) -> Result<Option<crate::models::purchase_order::PurchaseOrder>, crate::error::AppError>;
    async fn find_items(&self, order_id: i64) -> Result<Vec<crate::models::purchase_order::PurchaseOrderItem>, crate::error::AppError>;
    async fn list(&self, filter: &crate::dto::purchase_dto::PurchaseOrderFilterParams, params: &PaginationParams) -> Result<(Vec<crate::models::purchase_order::PurchaseOrder>, u64), crate::error::AppError>;
    async fn update_status(&self, id: i64, status: &str) -> Result<(), crate::error::AppError>;
    async fn reject(&self, id: i64, reason: &str) -> Result<(), crate::error::AppError>;
    async fn link_inbound(&self, order_id: i64, inbound_id: i64) -> Result<(), crate::error::AppError>;
    async fn delete(&self, id: i64) -> Result<(), crate::error::AppError>;
}

/// Sales order repository trait
#[async_trait]
pub trait SalesOrderRepository: Send + Sync {
    async fn save(&self, order: &crate::models::sales_order::SalesOrder, items: &[crate::dto::sales_dto::SalesOrderItemRequest]) -> Result<(), crate::error::AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<crate::models::sales_order::SalesOrder>, crate::error::AppError>;
    async fn find_by_order_no(&self, order_no: &str) -> Result<Option<crate::models::sales_order::SalesOrder>, crate::error::AppError>;
    async fn find_items(&self, order_id: i64) -> Result<Vec<crate::models::sales_order::SalesOrderItem>, crate::error::AppError>;
    async fn list(&self, filter: &crate::dto::sales_dto::SalesOrderFilterParams, params: &PaginationParams) -> Result<(Vec<crate::models::sales_order::SalesOrder>, u64), crate::error::AppError>;
    async fn update_status(&self, id: i64, status: &str) -> Result<(), crate::error::AppError>;
    async fn reject(&self, id: i64, reason: &str) -> Result<(), crate::error::AppError>;
    async fn link_outbound(&self, order_id: i64, outbound_id: i64) -> Result<(), crate::error::AppError>;
    async fn delete(&self, id: i64) -> Result<(), crate::error::AppError>;
}