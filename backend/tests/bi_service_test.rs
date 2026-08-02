//! BI integration tests — analytics aggregates over seeded data.

mod common;

use rust_decimal_macros::dec;
use steel_pipe_db::bi::services::BiService;

#[tokio::test]
async fn sales_trend_aggregates() {
    let pool = common::test_pool().await;
    // Seed sales orders across two statuses.
    sqlx::query(
        "INSERT INTO sales_orders (order_no, customer_id, order_date, status, total_amount) \
         VALUES ('SO-BI-1', 1, NOW(), 'approved', 10000), \
                ('SO-BI-2', 1, NOW(), 'approved', 20000), \
                ('SO-BI-3', 1, NOW(), 'draft', 5000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let trend = BiService::sales_trend(&pool, 1, 12).await.unwrap();
    let approved = trend.iter().find(|r| r.status == "approved").unwrap();
    assert_eq!(approved.order_count, 2);
    assert_eq!(approved.total_amount, dec!(30000));
}

#[tokio::test]
async fn inventory_value_counts_pipes() {
    let pool = common::test_pool().await;
    // Seed pipes directly.
    for i in 0..3 {
        sqlx::query(
            "INSERT INTO seamless_pipes (pipe_number, pipe_type, grade, od, wt, status) \
             VALUES ($1, 'casing', 'J55', 244.5, 11.05, 'in_stock')",
        )
        .bind(format!("BI-PIPE-{}", i))
        .execute(&pool)
        .await
        .unwrap();
    }
    let rows = BiService::inventory_value(&pool).await.unwrap();
    let seamless = rows.iter().find(|r| r.pipe_type == "seamless").unwrap();
    assert!(seamless.on_hand >= 3, "seeded pipes count as on-hand (got {})", seamless.on_hand);
}

#[tokio::test]
async fn finance_summary_counts() {
    let pool = common::test_pool().await;
    // Seed an open AR invoice + a posted journal entry + a payment.
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
         VALUES (1, 'JE-BI-1', CURRENT_DATE, 'posted')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let summary = BiService::finance_summary(&pool, 1).await.unwrap();
    assert_eq!(summary.posted_entries, 1);
    assert_eq!(summary.open_ar, dec!(100));
    assert_eq!(summary.open_ap, dec!(0));
}

#[tokio::test]
async fn supplier_performance_ranks() {
    let pool = common::test_pool().await;
    sqlx::query(
        "INSERT INTO suppliers (supplier_code, name) VALUES ('S-BI-1', '钢厂甲') ON CONFLICT DO NOTHING",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO purchase_orders (order_no, supplier_id, order_date, status, total_amount) \
         VALUES ('PO-BI-1', (SELECT id FROM suppliers WHERE supplier_code = 'S-BI-1'), NOW(), 'approved', 88000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let rows = BiService::supplier_performance(&pool, 1).await.unwrap();
    let mine = rows.iter().find(|r| r.supplier_name == "钢厂甲").unwrap();
    assert_eq!(mine.order_count, 1);
    assert_eq!(mine.order_total, dec!(88000));
}
