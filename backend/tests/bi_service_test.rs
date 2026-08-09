//! BI integration tests — analytics aggregates over seeded data.

mod common;

use erp_server::bi::services::BiService;

#[tokio::test]
async fn sales_trend_aggregates() {
    let pool = common::test_pool().await;
    // Seed sales orders across two statuses.
    sqlx::query(
        "INSERT INTO sales_orders (order_no, customer_id, order_date, status, total_amount) \
         VALUES ('SO-BI-1', 1, datetime('now'), 'approved', 10000), \
                ('SO-BI-2', 1, datetime('now'), 'approved', 20000), \
                ('SO-BI-3', 1, datetime('now'), 'draft', 5000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let trend = BiService::sales_trend(&pool, 1, 12).await.unwrap();
    let approved = trend.iter().find(|r| r.status == "approved").unwrap();
    assert_eq!(approved.order_count, 2);
    assert_eq!(approved.total_amount, 30000.0);
}

#[tokio::test]
async fn inventory_value_counts_on_hand() {
    let pool = common::test_pool().await;
    // Seed generic items + inbound movements (5 units each).
    for i in 0..3 {
        let item_id: i64 = sqlx::query_scalar(
            "INSERT INTO items (sku, name, category, unit, spec, price, status) \
             VALUES (?, ?, 'raw_material', 'pcs', NULL, 10.0, 'active') RETURNING id",
        )
        .bind(format!("BI-ITEM-{}", i))
        .bind(format!("测试商品 {}", i))
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO inventory_logs (item_id, quantity, change_type, created_at) \
             VALUES (?, 5, 'inbound', datetime('now'))",
        )
        .bind(item_id)
        .execute(&pool)
        .await
        .unwrap();
    }
    let rows = BiService::inventory_value(&pool).await.unwrap();
    assert!(
        rows.iter().any(|r| r.on_hand == 5.0),
        "seeded items must count as on-hand (got {:?})",
        rows
    );
}

#[tokio::test]
async fn finance_summary_counts() {
    let pool = common::test_pool().await;
    // Seed an open AR invoice + a posted journal entry.
    sqlx::query(
        "INSERT INTO finance_invoices \
         (tenant_id, invoice_no, invoice_type, party_id, amount, tax_amount, total_amount, status) \
         VALUES (1, 'INV-BI-1', 'sales', 1, 100, 0, 100, 'confirmed')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO journal_entries (tenant_id, entry_no, entry_date, status) \
         VALUES (1, 'JE-BI-1', date('now'), 'posted')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let summary = BiService::finance_summary(&pool, 1).await.unwrap();
    assert_eq!(summary.posted_entries, 1);
    assert_eq!(summary.open_ar, 100.0);
    assert_eq!(summary.open_ap, 0.0);
}

#[tokio::test]
async fn supplier_performance_ranks() {
    let pool = common::test_pool().await;
    sqlx::query(
        "INSERT INTO suppliers (supplier_code, name) VALUES ('S-BI-1', '供应商甲') ON CONFLICT DO NOTHING",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO purchase_orders (order_no, supplier_id, order_date, status, total_amount) \
         VALUES ('PO-BI-1', (SELECT id FROM suppliers WHERE supplier_code = 'S-BI-1'), datetime('now'), 'approved', 88000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let rows = BiService::supplier_performance(&pool, 1).await.unwrap();
    let mine = rows.iter().find(|r| r.supplier_name == "供应商甲").unwrap();
    assert_eq!(mine.order_count, 1);
    assert_eq!(mine.order_total, 88000.0);
}
