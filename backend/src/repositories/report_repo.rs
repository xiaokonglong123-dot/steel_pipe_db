use sqlx::SqlitePool;

use crate::error::AppError;

pub struct ReportRepo;

impl ReportRepo {
    pub async fn inventory_by_status(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT status, COUNT(*) as cnt FROM seamless_pipes \
             WHERE deleted_at IS NULL GROUP BY status ORDER BY cnt DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        let seamless_total: i64 = rows.iter().map(|(_, cnt)| cnt).sum();

        let mut result: Vec<serde_json::Value> = Vec::new();
        for (status, cnt) in rows {
            result.push(serde_json::json!({"status": status, "count": cnt}));
        }

        let screen_rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT status, COUNT(*) as cnt FROM screen_pipes \
             WHERE deleted_at IS NULL GROUP BY status ORDER BY cnt DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        let screen_total: i64 = screen_rows.iter().map(|(_, cnt)| cnt).sum();
        for (status, cnt) in screen_rows {
            result.push(serde_json::json!({"status": format!("screen_{}", status), "count": cnt}));
        }

        result.insert(
            0,
            serde_json::json!({"status": "total_seamless", "count": seamless_total}),
        );
        result.insert(
            1,
            serde_json::json!({"status": "total_screen", "count": screen_total}),
        );

        Ok(result)
    }

    pub async fn inventory_by_grade(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT grade, COUNT(*) as cnt FROM seamless_pipes \
             WHERE deleted_at IS NULL GROUP BY grade ORDER BY cnt DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        let screen_rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT base_grade, COUNT(*) as cnt FROM screen_pipes \
             WHERE deleted_at IS NULL GROUP BY base_grade ORDER BY cnt DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        let mut result: Vec<serde_json::Value> = Vec::new();
        for (grade, cnt) in rows {
            result.push(serde_json::json!({"grade": grade, "count": cnt, "pipe_type": "seamless"}));
        }
        for (grade, cnt) in screen_rows {
            result.push(serde_json::json!({"grade": grade, "count": cnt, "pipe_type": "screen"}));
        }
        Ok(result)
    }

    pub async fn inventory_by_type(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT pipe_type, COUNT(*) as cnt FROM seamless_pipes \
             WHERE deleted_at IS NULL GROUP BY pipe_type ORDER BY cnt DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        let screen_cnt: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM screen_pipes WHERE deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

        let mut result: Vec<serde_json::Value> = Vec::new();
        for (pt, cnt) in rows {
            result.push(serde_json::json!({"pipe_type": pt, "count": cnt}));
        }
        if screen_cnt.0 > 0 {
            result.push(serde_json::json!({"pipe_type": "screen", "count": screen_cnt.0}));
        }
        Ok(result)
    }

    pub async fn location_occupancy(pool: &SqlitePool) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, i64, i64, i64, String)> = sqlx::query_as(
            "SELECT l.full_code, l.max_capacity, l.current_usage, \
             (l.max_capacity - l.current_usage) as available, \
             CASE WHEN l.max_capacity > 0 THEN \
             CAST(ROUND(l.current_usage * 100.0 / l.max_capacity) AS INTEGER) \
             ELSE 0 END as occupancy_pct \
             FROM locations l WHERE l.is_active = 1 ORDER BY l.full_code",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(code, cap, usage, avail, pct)| {
                serde_json::json!({
                    "location": code,
                    "max_capacity": cap,
                    "current_usage": usage,
                    "available": avail,
                    "occupancy_pct": pct,
                })
            })
            .collect())
    }

    pub async fn purchase_order_report(
        pool: &SqlitePool,
        date_trunc: &str,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let group_expr = match date_trunc {
            "monthly" => "strftime('%Y-%m', order_date)",
            "quarterly" => {
                "strftime('%Y', order_date) || '-Q' || CAST((CAST(strftime('%m', order_date) AS INTEGER) + 2) / 3 AS TEXT)"
            }
            "yearly" => "strftime('%Y', order_date)",
            _ => "strftime('%Y-%m', order_date)",
        };

        let sql = format!(
            "SELECT {} as period, COUNT(*) as order_count, \
             COALESCE(SUM(total_amount), 0) as total_amount \
             FROM purchase_orders WHERE deleted_at IS NULL \
             GROUP BY period ORDER BY period DESC LIMIT 24",
            group_expr
        );

        let rows: Vec<(String, i64, f64)> = sqlx::query_as(&sql)
            .fetch_all(pool)
            .await
            .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(period, cnt, amount)| {
                serde_json::json!({
                    "period": period,
                    "order_count": cnt,
                    "total_amount": amount,
                })
            })
            .collect())
    }

    pub async fn sales_order_report(
        pool: &SqlitePool,
        date_trunc: &str,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let group_expr = match date_trunc {
            "monthly" => "strftime('%Y-%m', order_date)",
            "quarterly" => {
                "strftime('%Y', order_date) || '-Q' || CAST((CAST(strftime('%m', order_date) AS INTEGER) + 2) / 3 AS TEXT)"
            }
            "yearly" => "strftime('%Y', order_date)",
            _ => "strftime('%Y-%m', order_date)",
        };

        let sql = format!(
            "SELECT {} as period, COUNT(*) as order_count, \
             COALESCE(SUM(total_amount), 0) as total_amount \
             FROM sales_orders WHERE deleted_at IS NULL \
             GROUP BY period ORDER BY period DESC LIMIT 24",
            group_expr
        );

        let rows: Vec<(String, i64, f64)> = sqlx::query_as(&sql)
            .fetch_all(pool)
            .await
            .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(period, cnt, amount)| {
                serde_json::json!({
                    "period": period,
                    "order_count": cnt,
                    "total_amount": amount,
                })
            })
            .collect())
    }

    pub async fn order_status_distribution(
        pool: &SqlitePool,
        table: &str,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        // Whitelist table names to prevent SQL injection
        let allowed_tables = ["purchase_orders", "sales_orders"];
        if !allowed_tables.contains(&table) {
            return Err(AppError::BadRequest(format!("Invalid table: {}", table)));
        }
        let sql = format!(
            "SELECT status, COUNT(*) as cnt FROM {} \
             WHERE deleted_at IS NULL GROUP BY status ORDER BY cnt DESC",
            table
        );

        let rows: Vec<(String, i64)> = sqlx::query_as(&sql)
            .fetch_all(pool)
            .await
            .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(status, cnt)| serde_json::json!({"status": status, "count": cnt}))
            .collect())
    }

    pub async fn top_suppliers(
        pool: &SqlitePool,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, i64, f64)> = sqlx::query_as(
            "SELECT s.name, COUNT(*) as order_count, COALESCE(SUM(po.total_amount), 0) as total_amount \
             FROM purchase_orders po JOIN suppliers s ON po.supplier_id = s.id \
             WHERE po.deleted_at IS NULL AND s.deleted_at IS NULL \
             GROUP BY s.id, s.name ORDER BY total_amount DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(name, cnt, amount)| {
                serde_json::json!({
                    "supplier": name,
                    "order_count": cnt,
                    "total_amount": amount,
                })
            })
            .collect())
    }

    pub async fn top_customers(
        pool: &SqlitePool,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, i64, f64)> = sqlx::query_as(
            "SELECT c.name, COUNT(*) as order_count, COALESCE(SUM(so.total_amount), 0) as total_amount \
             FROM sales_orders so JOIN customers c ON so.customer_id = c.id \
             WHERE so.deleted_at IS NULL AND c.deleted_at IS NULL \
             GROUP BY c.id, c.name ORDER BY total_amount DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(name, cnt, amount)| {
                serde_json::json!({
                    "customer": name,
                    "order_count": cnt,
                    "total_amount": amount,
                })
            })
            .collect())
    }

    pub async fn quality_pass_fail_by_grade(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, i64, i64, String)> = sqlx::query_as(
            "SELECT sp.grade, \
             SUM(CASE WHEN qc.result = 'pass' THEN 1 ELSE 0 END) as pass_count, \
             SUM(CASE WHEN qc.result = 'fail' THEN 1 ELSE 0 END) as fail_count, \
             'seamless' as pipe_type \
             FROM quality_certs qc JOIN seamless_pipes sp ON qc.pipe_id = sp.id AND qc.pipe_type = 'seamless' \
             WHERE sp.deleted_at IS NULL GROUP BY sp.grade \
             UNION ALL \
             SELECT sp.base_grade, \
             SUM(CASE WHEN qc.result = 'pass' THEN 1 ELSE 0 END), \
             SUM(CASE WHEN qc.result = 'fail' THEN 1 ELSE 0 END), \
             'screen' \
             FROM quality_certs qc JOIN screen_pipes sp ON qc.pipe_id = sp.id AND qc.pipe_type = 'screen' \
             WHERE sp.deleted_at IS NULL GROUP BY sp.base_grade \
             ORDER BY pipe_type, grade",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(grade, pass, fail, pipe_type)| {
                let total = pass + fail;
                let pass_rate = if total > 0 {
                    format!("{:.1}%", pass as f64 * 100.0 / total as f64)
                } else {
                    "N/A".into()
                };
                serde_json::json!({
                    "grade": grade,
                    "pipe_type": pipe_type,
                    "pass_count": pass,
                    "fail_count": fail,
                    "total": total,
                    "pass_rate": pass_rate,
                })
            })
            .collect())
    }

    pub async fn quality_certs_by_month(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, i64, i64, i64)> = sqlx::query_as(
            "SELECT strftime('%Y-%m', inspect_date) as month, \
             COUNT(*) as total, \
             SUM(CASE WHEN result = 'pass' THEN 1 ELSE 0 END) as passed, \
             SUM(CASE WHEN result = 'fail' THEN 1 ELSE 0 END) as failed \
             FROM quality_certs WHERE inspect_date IS NOT NULL \
             GROUP BY month ORDER BY month DESC LIMIT 12",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(month, total, passed, failed)| {
                let pass_rate = if total > 0 {
                    format!("{:.1}%", passed as f64 * 100.0 / total as f64)
                } else {
                    "0%".into()
                };
                serde_json::json!({
                    "month": month,
                    "total": total,
                    "passed": passed,
                    "failed": failed,
                    "pass_rate": pass_rate,
                })
            })
            .collect())
    }

    pub async fn total_stock(pool: &SqlitePool) -> Result<serde_json::Value, AppError> {
        let seamless: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM seamless_pipes WHERE deleted_at IS NULL AND status = 'in_stock'",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

        let screen: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM screen_pipes WHERE deleted_at IS NULL AND status = 'in_stock'",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

        Ok(serde_json::json!({
            "seamless_pipes": seamless.0,
            "screen_pipes": screen.0,
            "total": seamless.0 + screen.0,
        }))
    }

    pub async fn recent_inbound(
        pool: &SqlitePool,
        days: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT ir.record_no, ir.inbound_type, ir.approval_status, ir.created_at \
             FROM inbound_records ir \
             WHERE ir.created_at >= datetime('now', ? || ' days') \
             ORDER BY ir.created_at DESC LIMIT ?",
        )
        .bind(format!("-{}", days))
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(no, ty, status, at)| {
                serde_json::json!({
                    "record_no": no,
                    "type": ty,
                    "approval_status": status,
                    "created_at": at,
                })
            })
            .collect())
    }

    pub async fn recent_outbound(
        pool: &SqlitePool,
        days: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT orr.record_no, orr.outbound_type, orr.approval_status, orr.created_at \
             FROM outbound_records orr \
             WHERE orr.created_at >= datetime('now', ? || ' days') \
             ORDER BY orr.created_at DESC LIMIT ?",
        )
        .bind(format!("-{}", days))
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(no, ty, status, at)| {
                serde_json::json!({
                    "record_no": no,
                    "type": ty,
                    "approval_status": status,
                    "created_at": at,
                })
            })
            .collect())
    }

    pub async fn inbound_count_30d(pool: &SqlitePool) -> Result<i64, AppError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM inbound_records \
             WHERE created_at >= datetime('now', '-30 days')",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        Ok(row.0)
    }

    pub async fn outbound_count_30d(pool: &SqlitePool) -> Result<i64, AppError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM outbound_records \
             WHERE created_at >= datetime('now', '-30 days')",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        Ok(row.0)
    }

    pub async fn pending_approvals(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let mut result: Vec<serde_json::Value> = Vec::new();

        let inbound: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT id, record_no, 'inbound' as ref_type FROM inbound_records \
             WHERE approval_status = 'pending' ORDER BY created_at DESC LIMIT 20",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id, no, ref_type) in inbound {
            result.push(serde_json::json!({
                "id": id,
                "reference_no": no,
                "reference_type": ref_type,
            }));
        }

        let outbound: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT id, record_no, 'outbound' as ref_type FROM outbound_records \
             WHERE approval_status = 'pending' ORDER BY created_at DESC LIMIT 20",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id, no, ref_type) in outbound {
            result.push(serde_json::json!({
                "id": id,
                "reference_no": no,
                "reference_type": ref_type,
            }));
        }

        let purchase: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT id, order_no, 'purchase_order' as ref_type FROM purchase_orders \
             WHERE status = 'pending' AND deleted_at IS NULL ORDER BY created_at DESC LIMIT 20",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id, no, ref_type) in purchase {
            result.push(serde_json::json!({
                "id": id,
                "reference_no": no,
                "reference_type": ref_type,
            }));
        }

        let sales: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT id, order_no, 'sales_order' as ref_type FROM sales_orders \
             WHERE status = 'pending' AND deleted_at IS NULL ORDER BY created_at DESC LIMIT 20",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id, no, ref_type) in sales {
            result.push(serde_json::json!({
                "id": id,
                "reference_no": no,
                "reference_type": ref_type,
            }));
        }

        Ok(result)
    }

    pub async fn recent_quality_failures(
        pool: &SqlitePool,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
            "SELECT qc.cert_no, qc.pipe_type, qc.pipe_id, qc.inspect_date, qc.notes \
             FROM quality_certs qc \
             WHERE qc.result = 'fail' \
             ORDER BY qc.created_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(cert_no, pipe_type, pipe_id, inspect_date, notes)| {
                serde_json::json!({
                    "cert_no": cert_no,
                    "pipe_type": pipe_type,
                    "pipe_id": pipe_id,
                    "inspect_date": inspect_date,
                    "notes": notes,
                })
            })
            .collect())
    }

    pub async fn pending_approval_count(pool: &SqlitePool) -> Result<i64, AppError> {
        let ib: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM inbound_records WHERE approval_status = 'pending'",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

        let ob: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM outbound_records WHERE approval_status = 'pending'",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

        Ok(ib.0 + ob.0)
    }
}
