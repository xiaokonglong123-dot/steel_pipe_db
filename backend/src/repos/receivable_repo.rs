use sqlx::{Executor, Sqlite, SqlitePool};

use crate::error::AppError;
use crate::repos::inventory_repo::{InboundOrderRow, OutboundOrderRow};

pub async fn insert_inbound<'e, E>(
    executor: E,
    record_no: &str,
    order_id: i64,
    supplier_id: i64,
    created_by: i64,
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = Sqlite>,
{
    let result = sqlx::query(
        "INSERT INTO inbound_records
            (record_no, inbound_type, order_id, supplier_id, status, created_by)
         VALUES (?, 'purchase', ?, ?, 'posted', ?)",
    )
    .bind(record_no)
    .bind(order_id)
    .bind(supplier_id)
    .bind(created_by)
    .execute(executor)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn insert_inbound_item<'e, E>(
    executor: E,
    record_id: i64,
    item_id: i64,
    location_id: i64,
    quantity: f64,
) -> Result<(), AppError>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query(
        "INSERT INTO inbound_items (record_id, item_id, location_id, quantity)
         VALUES (?, ?, ?, ?)",
    )
    .bind(record_id)
    .bind(item_id)
    .bind(location_id)
    .bind(quantity)
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn insert_outbound<'e, E>(
    executor: E,
    record_no: &str,
    order_id: i64,
    customer_id: i64,
    created_by: i64,
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = Sqlite>,
{
    let result = sqlx::query(
        "INSERT INTO outbound_records
            (record_no, outbound_type, order_id, customer_id, status, created_by)
         VALUES (?, 'sales', ?, ?, 'posted', ?)",
    )
    .bind(record_no)
    .bind(order_id)
    .bind(customer_id)
    .bind(created_by)
    .execute(executor)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn insert_outbound_item<'e, E>(
    executor: E,
    record_id: i64,
    item_id: i64,
    location_id: i64,
    quantity: f64,
) -> Result<(), AppError>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query(
        "INSERT INTO outbound_items (record_id, item_id, location_id, quantity)
         VALUES (?, ?, ?, ?)",
    )
    .bind(record_id)
    .bind(item_id)
    .bind(location_id)
    .bind(quantity)
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn find_inbound(pool: &SqlitePool, id: i64) -> Result<Option<InboundOrderRow>, AppError> {
    crate::repos::inventory_repo::get_inbound_order_by_id(pool, id).await
}

pub async fn find_outbound(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<OutboundOrderRow>, AppError> {
    crate::repos::inventory_repo::get_outbound_order_by_id(pool, id).await
}
