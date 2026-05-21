use sqlx::{Row, SqlitePool};

use crate::error::AppError;

pub struct DataIORepo;

impl DataIORepo {
    pub async fn export_seamless_pipes(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows = sqlx::query(
            "SELECT id, pipe_number, batch_number, pipe_type, grade, od, wt, length, weight_per_unit, \
             end_type, coupling_type, coupling_od, coupling_length, heat_number, serial_number, \
             manufacturer, production_date, cert_number, location_id, status, notes \
             FROM seamless_pipes WHERE deleted_at IS NULL ORDER BY id"
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows.iter().map(|r| serde_json::json!({
            "id": r.get::<i64, _>("id"),
            "pipe_number": r.get::<String, _>("pipe_number"),
            "batch_number": r.get::<Option<String>, _>("batch_number"),
            "pipe_type": r.get::<String, _>("pipe_type"),
            "grade": r.get::<String, _>("grade"),
            "od": r.get::<f64, _>("od"),
            "wt": r.get::<f64, _>("wt"),
            "length": r.get::<Option<f64>, _>("length"),
            "weight_per_unit": r.get::<Option<f64>, _>("weight_per_unit"),
            "end_type": r.get::<Option<String>, _>("end_type"),
            "coupling_type": r.get::<Option<String>, _>("coupling_type"),
            "coupling_od": r.get::<Option<f64>, _>("coupling_od"),
            "coupling_length": r.get::<Option<f64>, _>("coupling_length"),
            "heat_number": r.get::<Option<String>, _>("heat_number"),
            "serial_number": r.get::<Option<String>, _>("serial_number"),
            "manufacturer": r.get::<Option<String>, _>("manufacturer"),
            "production_date": r.get::<Option<String>, _>("production_date"),
            "cert_number": r.get::<Option<String>, _>("cert_number"),
            "location_id": r.get::<Option<i64>, _>("location_id"),
            "status": r.get::<String, _>("status"),
            "notes": r.get::<Option<String>, _>("notes"),
        })).collect())
    }

    pub async fn export_screen_pipes(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows = sqlx::query(
            "SELECT id, pipe_number, batch_number, screen_type, slot_size, filtration_grade, \
             base_od, base_wt, base_grade, base_end_type, length, weight_per_unit, heat_number, \
             serial_number, manufacturer, production_date, cert_number, location_id, status, notes \
             FROM screen_pipes WHERE deleted_at IS NULL ORDER BY id"
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows.iter().map(|r| serde_json::json!({
            "id": r.get::<i64, _>("id"),
            "pipe_number": r.get::<String, _>("pipe_number"),
            "batch_number": r.get::<Option<String>, _>("batch_number"),
            "screen_type": r.get::<String, _>("screen_type"),
            "slot_size": r.get::<Option<f64>, _>("slot_size"),
            "filtration_grade": r.get::<Option<String>, _>("filtration_grade"),
            "base_od": r.get::<f64, _>("base_od"),
            "base_wt": r.get::<f64, _>("base_wt"),
            "base_grade": r.get::<String, _>("base_grade"),
            "base_end_type": r.get::<Option<String>, _>("base_end_type"),
            "length": r.get::<Option<f64>, _>("length"),
            "weight_per_unit": r.get::<Option<f64>, _>("weight_per_unit"),
            "heat_number": r.get::<Option<String>, _>("heat_number"),
            "serial_number": r.get::<Option<String>, _>("serial_number"),
            "manufacturer": r.get::<Option<String>, _>("manufacturer"),
            "production_date": r.get::<Option<String>, _>("production_date"),
            "cert_number": r.get::<Option<String>, _>("cert_number"),
            "location_id": r.get::<Option<i64>, _>("location_id"),
            "status": r.get::<String, _>("status"),
            "notes": r.get::<Option<String>, _>("notes"),
        })).collect())
    }

    pub async fn export_inventory(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let seamless: Vec<(i64, String, String, String, f64, f64, String, Option<i64>, Option<String>)> = sqlx::query_as(
            "SELECT sp.id, sp.pipe_number, 'seamless', sp.grade, sp.od, sp.wt, sp.status, sp.location_id, sp.heat_number \
             FROM seamless_pipes sp WHERE sp.deleted_at IS NULL AND sp.status != 'outbound'"
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        let screen: Vec<(i64, String, String, String, f64, f64, String, Option<i64>, Option<String>)> = sqlx::query_as(
            "SELECT sp.id, sp.pipe_number, 'screen', sp.base_grade, sp.base_od, sp.base_wt, sp.status, sp.location_id, sp.heat_number \
             FROM screen_pipes sp WHERE sp.deleted_at IS NULL AND sp.status != 'outbound'"
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        let mut result = Vec::new();
        for r in seamless {
            result.push(serde_json::json!({
                "pipe_id": r.0,
                "pipe_number": r.1,
                "pipe_type": r.2,
                "grade": r.3,
                "od": r.4,
                "wt": r.5,
                "status": r.6,
                "location_id": r.7,
                "heat_number": r.8,
            }));
        }
        for r in screen {
            result.push(serde_json::json!({
                "pipe_id": r.0,
                "pipe_number": r.1,
                "pipe_type": r.2,
                "grade": r.3,
                "od": r.4,
                "wt": r.5,
                "status": r.6,
                "location_id": r.7,
                "heat_number": r.8,
            }));
        }
        Ok(result)
    }

    pub async fn export_purchase_orders(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(i64, String, i64, String, String, Option<f64>, Option<String>, Option<i64>)> = sqlx::query_as(
            "SELECT id, order_no, supplier_id, order_date, status, total_amount, notes, created_by \
             FROM purchase_orders WHERE deleted_at IS NULL ORDER BY id"
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        let mut result = Vec::new();
        for (id, order_no, supplier_id, order_date, status, total_amount, notes, created_by) in rows {
            let count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM purchase_order_items WHERE order_id = ?"
            )
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(AppError::from)?;

            result.push(serde_json::json!({
                "id": id,
                "order_no": order_no,
                "supplier_id": supplier_id,
                "order_date": order_date,
                "status": status,
                "total_amount": total_amount,
                "notes": notes,
                "created_by": created_by,
                "items_count": count.0,
            }));
        }
        Ok(result)
    }

    pub async fn export_sales_orders(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(i64, String, i64, String, String, Option<f64>, Option<String>, Option<i64>)> = sqlx::query_as(
            "SELECT id, order_no, customer_id, order_date, status, total_amount, notes, created_by \
             FROM sales_orders WHERE deleted_at IS NULL ORDER BY id"
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        let mut result = Vec::new();
        for (id, order_no, customer_id, order_date, status, total_amount, notes, created_by) in rows {
            let count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM sales_order_items WHERE order_id = ?"
            )
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(AppError::from)?;

            result.push(serde_json::json!({
                "id": id,
                "order_no": order_no,
                "customer_id": customer_id,
                "order_date": order_date,
                "status": status,
                "total_amount": total_amount,
                "notes": notes,
                "created_by": created_by,
                "items_count": count.0,
            }));
        }
        Ok(result)
    }

    pub async fn export_quality_certs(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(i64, String, String, i64, Option<String>, String, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT id, cert_number, pipe_type, pipe_id, cert_date, result, inspector, inspection_body, notes \
             FROM quality_certs WHERE deleted_at IS NULL ORDER BY id"
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows.into_iter().map(|r| serde_json::json!({
            "id": r.0,
            "cert_number": r.1,
            "pipe_type": r.2,
            "pipe_id": r.3,
            "cert_date": r.4,
            "result": r.5,
            "inspector": r.6,
            "inspection_body": r.7,
            "notes": r.8,
        })).collect())
    }

    pub async fn import_seamless_pipes(
        pool: &SqlitePool,
        rows: &[serde_json::Value],
    ) -> Result<(u64, Vec<String>), AppError> {
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for row in rows {
            let pipe_number = match row.get("pipe_number").and_then(|v| v.as_str()) {
                Some(v) => v.to_string(),
                None => {
                    errors.push("Missing pipe_number field".into());
                    continue;
                }
            };

            let exists: Option<(i64,)> = sqlx::query_as(
                "SELECT id FROM seamless_pipes WHERE pipe_number = ? AND deleted_at IS NULL"
            )
            .bind(&pipe_number)
            .fetch_optional(pool)
            .await
            .map_err(AppError::from)?;

            if exists.is_some() {
                errors.push(format!("Pipe number {} already exists", pipe_number));
                continue;
            }

            let pipe_type = row.get("pipe_type").and_then(|v| v.as_str()).unwrap_or("casing");
            let grade = row.get("grade").and_then(|v| v.as_str()).unwrap_or("J55");
            let od = row.get("od").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let wt = row.get("wt").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let status = row.get("status").and_then(|v| v.as_str()).unwrap_or("in_stock");

            sqlx::query(
                "INSERT INTO seamless_pipes (pipe_number, batch_number, pipe_type, grade, od, wt, \
                 length, weight_per_unit, end_type, coupling_type, coupling_od, coupling_length, \
                 heat_number, serial_number, manufacturer, production_date, cert_number, status, notes) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&pipe_number)
            .bind(row.get("batch_number").and_then(|v| v.as_str()))
            .bind(pipe_type)
            .bind(grade)
            .bind(od)
            .bind(wt)
            .bind(row.get("length").and_then(|v| v.as_f64()))
            .bind(row.get("weight_per_unit").and_then(|v| v.as_f64()))
            .bind(row.get("end_type").and_then(|v| v.as_str()))
            .bind(row.get("coupling_type").and_then(|v| v.as_str()))
            .bind(row.get("coupling_od").and_then(|v| v.as_f64()))
            .bind(row.get("coupling_length").and_then(|v| v.as_f64()))
            .bind(row.get("heat_number").and_then(|v| v.as_str()))
            .bind(row.get("serial_number").and_then(|v| v.as_str()))
            .bind(row.get("manufacturer").and_then(|v| v.as_str()))
            .bind(row.get("production_date").and_then(|v| v.as_str()))
            .bind(row.get("cert_number").and_then(|v| v.as_str()))
            .bind(status)
            .bind(row.get("notes").and_then(|v| v.as_str()))
            .execute(pool)
            .await
            .map_err(|e| AppError::ImportError(format!("Failed to insert {}: {}", pipe_number, e)))?;

            imported += 1;
        }

        Ok((imported, errors))
    }

    pub async fn import_screen_pipes(
        pool: &SqlitePool,
        rows: &[serde_json::Value],
    ) -> Result<(u64, Vec<String>), AppError> {
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for row in rows {
            let pipe_number = match row.get("pipe_number").and_then(|v| v.as_str()) {
                Some(v) => v.to_string(),
                None => {
                    errors.push("Missing pipe_number field".into());
                    continue;
                }
            };

            let exists: Option<(i64,)> = sqlx::query_as(
                "SELECT id FROM screen_pipes WHERE pipe_number = ? AND deleted_at IS NULL"
            )
            .bind(&pipe_number)
            .fetch_optional(pool)
            .await
            .map_err(AppError::from)?;

            if exists.is_some() {
                errors.push(format!("Pipe number {} already exists", pipe_number));
                continue;
            }

            let base_od = row.get("base_od").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let base_wt = row.get("base_wt").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let screen_type = row.get("screen_type").and_then(|v| v.as_str()).unwrap_or("wire_wrapped");
            let base_grade = row.get("base_grade").and_then(|v| v.as_str()).unwrap_or("J55");
            let status = row.get("status").and_then(|v| v.as_str()).unwrap_or("in_stock");

            sqlx::query(
                "INSERT INTO screen_pipes (pipe_number, batch_number, screen_type, slot_size, \
                 filtration_grade, base_od, base_wt, base_grade, base_end_type, length, \
                 weight_per_unit, heat_number, serial_number, manufacturer, production_date, \
                 cert_number, status, notes) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&pipe_number)
            .bind(row.get("batch_number").and_then(|v| v.as_str()))
            .bind(screen_type)
            .bind(row.get("slot_size").and_then(|v| v.as_f64()))
            .bind(row.get("filtration_grade").and_then(|v| v.as_str()))
            .bind(base_od)
            .bind(base_wt)
            .bind(base_grade)
            .bind(row.get("base_end_type").and_then(|v| v.as_str()))
            .bind(row.get("length").and_then(|v| v.as_f64()))
            .bind(row.get("weight_per_unit").and_then(|v| v.as_f64()))
            .bind(row.get("heat_number").and_then(|v| v.as_str()))
            .bind(row.get("serial_number").and_then(|v| v.as_str()))
            .bind(row.get("manufacturer").and_then(|v| v.as_str()))
            .bind(row.get("production_date").and_then(|v| v.as_str()))
            .bind(row.get("cert_number").and_then(|v| v.as_str()))
            .bind(status)
            .bind(row.get("notes").and_then(|v| v.as_str()))
            .execute(pool)
            .await
            .map_err(|e| AppError::ImportError(format!("Failed to insert {}: {}", pipe_number, e)))?;

            imported += 1;
        }

        Ok((imported, errors))
    }
}
