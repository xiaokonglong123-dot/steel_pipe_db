use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::ApiResponse;
use crate::services::receipt_service::{self, ReceivedItemInput};

#[derive(Deserialize)]
pub struct ReceiveRequest {
    pub items: Vec<ReceivedItemDto>,
}

#[derive(Deserialize)]
pub struct ReceivedItemDto {
    pub item_id: i64,
    pub location_id: i64,
    pub quantity: f64,
}

pub async fn receive_purchase_order(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<ReceiveRequest>,
) -> Result<impl IntoResponse, AppError> {
    let items = req
        .items
        .into_iter()
        .map(|item| ReceivedItemInput {
            item_id: item.item_id,
            location_id: item.location_id,
            quantity: item.quantity,
        })
        .collect::<Vec<_>>();
    let record = receipt_service::receive_purchase_order(&pool, id, &items, &user).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::created(record))))
}
