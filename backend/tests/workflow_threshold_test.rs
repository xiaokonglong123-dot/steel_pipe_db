use erp_v2::middleware::auth::AuthUser;
use erp_v2::repos::workflow_repo;
use erp_v2::services::workflow_service;
use rust_decimal::Decimal;
use sqlx::SqlitePool;

mod common;

async fn bootstrap_admin(pool: &SqlitePool) {
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO users (id, username, display_name, password_hash, is_active)
         VALUES (1, 'admin', 'Administrator', '$argon2id$v=19$m=19456,t=2,p=1$YWFhYQ$hello', 1)",
    )
    .execute(pool)
    .await
    .unwrap();
}

fn admin() -> AuthUser {
    AuthUser {
        id: 1,
        username: "admin".into(),
        display_name: "Administrator".into(),
        permissions: vec!["order.approve".into()],
    }
}

async fn seed_amount_threshold_workflow(pool: &SqlitePool) -> i64 {
    let _ = sqlx::query("UPDATE workflows SET is_active = 0 WHERE applies_to = 'purchase_order'")
        .execute(pool).await.unwrap();

    let wf_id: i64 = sqlx::query_scalar(
        "INSERT INTO workflows (name, applies_to, is_active) VALUES ('金额规则审批', 'purchase_order', 1) RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap();

    let s_draft = sqlx::query_scalar::<_, i64>(
        "INSERT INTO workflow_states (workflow_id, state_key, doc_status, is_initial, is_final) VALUES (?, 'draft', 0, 1, 0) RETURNING id",
    )
    .bind(wf_id)
    .fetch_one(pool).await.unwrap();
    let s_submitted: i64 = sqlx::query_scalar(
        "INSERT INTO workflow_states (workflow_id, state_key, doc_status, is_initial, is_final) VALUES (?, 'submitted', 1, 0, 0) RETURNING id",
    )
    .bind(wf_id).fetch_one(pool).await.unwrap();
    let s_senior: i64 = sqlx::query_scalar(
        "INSERT INTO workflow_states (workflow_id, state_key, doc_status, is_initial, is_final) VALUES (?, 'senior_review', 2, 0, 0) RETURNING id",
    )
    .bind(wf_id).fetch_one(pool).await.unwrap();
    let s_approved: i64 = sqlx::query_scalar(
        "INSERT INTO workflow_states (workflow_id, state_key, doc_status, is_initial, is_final) VALUES (?, 'approved', 3, 0, 1) RETURNING id",
    )
    .bind(wf_id).fetch_one(pool).await.unwrap();
    let s_rejected: i64 = sqlx::query_scalar(
        "INSERT INTO workflow_states (workflow_id, state_key, doc_status, is_initial, is_final) VALUES (?, 'rejected', 4, 0, 1) RETURNING id",
    )
    .bind(wf_id).fetch_one(pool).await.unwrap();

    sqlx::query(
        "INSERT INTO workflow_transitions (workflow_id, from_state_id, to_state_id, action, required_role, is_auto, amount_threshold)
         VALUES (?, ?, ?, 'submit', NULL, 0, NULL)",
    ).bind(wf_id).bind(s_draft).bind(s_submitted).execute(pool).await.unwrap();

    sqlx::query(
        "INSERT INTO workflow_transitions (workflow_id, from_state_id, to_state_id, action, required_role, is_auto, amount_threshold)
         VALUES (?, ?, ?, 'approve', NULL, 0, NULL)",
    ).bind(wf_id).bind(s_submitted).bind(s_approved).execute(pool).await.unwrap();

    sqlx::query(
        "INSERT INTO workflow_transitions (workflow_id, from_state_id, to_state_id, action, required_role, is_auto, amount_threshold)
         VALUES (?, ?, ?, 'approve', 'order.approve', 0, '10000')",
    ).bind(wf_id).bind(s_submitted).bind(s_senior).execute(pool).await.unwrap();

    sqlx::query(
        "INSERT INTO workflow_transitions (workflow_id, from_state_id, to_state_id, action, required_role, is_auto, amount_threshold)
         VALUES (?, ?, ?, 'approve', NULL, 0, NULL)",
    ).bind(wf_id).bind(s_senior).bind(s_approved).execute(pool).await.unwrap();

    sqlx::query(
        "INSERT INTO workflow_transitions (workflow_id, from_state_id, to_state_id, action, required_role, is_auto, amount_threshold)
         VALUES (?, ?, ?, 'reject', NULL, 0, NULL)",
    ).bind(wf_id).bind(s_submitted).bind(s_rejected).execute(pool).await.unwrap();

    wf_id
}

#[tokio::test]
async fn low_amount_skips_senior_review_directly_approved() {
    let (pool, _dir) = common::test_pool().await;
    bootstrap_admin(&pool).await;
    let _wf_id = seed_amount_threshold_workflow(&pool).await;
    let user = admin();

    let inst = workflow_service::start_instance(&pool, "purchase_order", 1, &user).await.unwrap();

    workflow_service::transition_with_amount(&pool, inst.id, "submit", &user, None, Some(Decimal::new(5000, 0))).await.unwrap();
    let inst = workflow_repo::find_instance_by_id(&pool, inst.id).await.unwrap().unwrap();
    assert_eq!(inst.current_state, "submitted");

    workflow_service::transition_with_amount(&pool, inst.id, "approve", &user, None, Some(Decimal::new(5000, 0))).await.unwrap();
    let inst = workflow_repo::find_instance_by_id(&pool, inst.id).await.unwrap().unwrap();
    assert_eq!(inst.current_state, "approved", "金额 5000 < 10000 应直接 approve 跳过 senior_review");
    assert_eq!(inst.status, "completed");
}

#[tokio::test]
async fn high_amount_triggers_senior_review_then_approve() {
    let (pool, _dir) = common::test_pool().await;
    bootstrap_admin(&pool).await;
    let _ = seed_amount_threshold_workflow(&pool).await;
    let user = admin();

    let inst = workflow_service::start_instance(&pool, "purchase_order", 2, &user).await.unwrap();

    workflow_service::transition_with_amount(&pool, inst.id, "submit", &user, None, Some(Decimal::new(15000, 0))).await.unwrap();

    workflow_service::transition_with_amount(&pool, inst.id, "approve", &user, None, Some(Decimal::new(15000, 0))).await.unwrap();
    let inst = workflow_repo::find_instance_by_id(&pool, inst.id).await.unwrap().unwrap();
    assert_eq!(inst.current_state, "senior_review", "金额 15000 ≥ 10000 应触发 senior_review");
    assert_eq!(inst.status, "active", "实例仍应 active 等待二级审批");

    workflow_service::transition_with_amount(&pool, inst.id, "approve", &user, None, Some(Decimal::new(15000, 0))).await.unwrap();
    let inst = workflow_repo::find_instance_by_id(&pool, inst.id).await.unwrap().unwrap();
    assert_eq!(inst.current_state, "approved", "二级审批后 approvd");
    assert_eq!(inst.status, "completed");
}
