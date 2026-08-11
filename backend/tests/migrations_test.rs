//! migrations 通过且种子数据可查

mod common;

use common::test_pool;

#[tokio::test]
async fn migrate_and_seed() {
    let (pool, _dir) = test_pool().await;

    let role_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM roles")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(role_count, 6, "应为 6 个种子角色");

    let perm_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM permissions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(perm_count, 11, "应为 11 个权限");

    let admin_perms: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM role_permissions WHERE role_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(admin_perms, 11, "admin 应有全部 11 个权限");

    // 各业务表存在且空
    for table in [
        "items",
        "suppliers",
        "customers",
        "locations",
        "inventory",
        "inventory_logs",
        "inbound_records",
        "outbound_records",
        "purchase_orders",
        "sales_orders",
        "accounts",
        "journal_entries",
        "workflow_instances",
    ] {
        let n: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&pool)
            .await
            .unwrap_or_else(|e| panic!("table {table} query failed: {e}"));
        assert_eq!(n, 0, "table {table} should be empty after migration");
    }

    let wf_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workflows")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(wf_count, 2, "应为 2 条种子 workflow（PO+SO）");
    let ws_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workflow_states")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(ws_count, 8, "应为 8 个 workflow_states（PO+SO 各 4）");
    let wt_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workflow_transitions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(wt_count, 6, "应为 6 个 workflow_transitions（PO+SO 各 3）");
}
