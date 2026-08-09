//! Item (商品) service — CRUD + search for the items master.

use sqlx::SqlitePool;

use crate::dto::item_dto::{CreateItemRequest, ItemFilter, ItemSkuQuery, UpdateItemRequest};
use crate::error::AppError;
use crate::models::item::Item;
use crate::repositories::item_repo::ItemRepo;

pub struct ItemService;

impl ItemService {
    /// Creates an item. Rejects duplicate SKUs.
    pub async fn create_item(
        pool: &SqlitePool,
        dto: &CreateItemRequest,
    ) -> Result<Item, AppError> {
        let sku = dto.sku.trim();
        if sku.is_empty() {
            return Err(AppError::Validation("SKU is required".into()));
        }
        if ItemRepo::sku_exists(pool, sku).await.map_err(AppError::from)? {
            return Err(AppError::Validation(format!(
                "Item SKU '{}' already exists",
                sku
            )));
        }
        ItemRepo::create(pool, dto).await.map_err(AppError::from)
    }

    /// Gets a single item by ID.
    pub async fn get_item(pool: &SqlitePool, id: i64) -> Result<Item, AppError> {
        ItemRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Item id={} not found", id)))
    }

    /// Updates an item by ID.
    pub async fn update_item(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateItemRequest,
    ) -> Result<Item, AppError> {
        Self::get_item(pool, id).await?;
        ItemRepo::update(pool, id, dto).await.map_err(AppError::from)
    }

    /// Soft-deletes an item by ID.
    pub async fn delete_item(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        Self::get_item(pool, id).await?;
        ItemRepo::delete(pool, id).await.map_err(AppError::from)
    }

    /// Paginated item list.
    pub async fn list_items(
        pool: &SqlitePool,
        filter: &ItemFilter,
    ) -> Result<(Vec<Item>, u64), AppError> {
        ItemRepo::list(pool, filter).await.map_err(AppError::from)
    }

    /// Search items by SKU (partial match) — returns up to 20 active items.
    pub async fn search_by_sku(
        pool: &SqlitePool,
        query: &ItemSkuQuery,
    ) -> Result<Vec<Item>, AppError> {
        let filter = ItemFilter {
            page: Some(1),
            page_size: Some(20),
            q: Some(format!("%{}%", query.sku.trim())),
            category: None,
            status: Some("active".into()),
        };
        let (items, _) = ItemRepo::list(pool, &filter).await.map_err(AppError::from)?;
        Ok(items)
    }
}
