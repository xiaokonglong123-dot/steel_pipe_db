use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::ApiResponse;
use crate::services::shipment_service::{self, ShippedItemInput};

#[derive(Deserialize)]
pub struct ShipRequest {
    pub items: Vec<ShippedItemDto>,
}

#[derive(Deserialize)]
pub struct ShippedItemDto {
    pub item_id: i64,
    pub location_id: i64,
    pub quantity: f64,
}

pub async fn ship_sales_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<ShipRequest>,
) -> Result<impl IntoResponse, AppError> {
    let items = req
        .items
        .into_iter()
        .map(|item| ShippedItemInput {
            item_id: item.item_id,
            location_id: item.location_id,
            quantity: item.quantity,
        })
        .collect::<Vec<_>>();
    let record = shipment_service::ship_sales_order(&pool, id, &items, &user).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::created(record))))
}
