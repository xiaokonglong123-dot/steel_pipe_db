//! Notification integration tests — send, list, read, preferences, templates.

mod common;

use erp_server::dto::notification_dto::{
    CreateTemplateRequest, SendNotificationRequest, UpdatePreferenceRequest,
};
use erp_server::notification::services::NotificationService;

/// Seed a user (id 1) for notification FK.
async fn seed_user(pool: &sqlx::SqlitePool) {
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, display_name, role, tenant_id) \
         VALUES (1, 'notif_user', 'x', '通知用户', 'admin', 1) ON CONFLICT (id) DO NOTHING",
    )
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn send_and_list_notifications() {
    let pool = common::test_pool().await;
    seed_user(&pool).await;
    let n = NotificationService::send(
        &pool,
        1,
        &SendNotificationRequest {
            user_id: 1,
            title: "采购订单已审批".into(),
            content: Some("PO-1 已通过".into()),
            notify_type: Some("workflow".into()),
        },
    )
    .await
    .unwrap();
    assert!(!n.is_read);

    let list = NotificationService::list(&pool, 1, 1, true).await.unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn mark_read_and_unread_count() {
    let pool = common::test_pool().await;
    seed_user(&pool).await;
    NotificationService::send(&pool, 1, &SendNotificationRequest { user_id: 1, title: "A".into(), content: None, notify_type: None }).await.unwrap();
    let n2 = NotificationService::send(&pool, 1, &SendNotificationRequest { user_id: 1, title: "B".into(), content: None, notify_type: None }).await.unwrap();

    let count = NotificationService::unread_count(&pool, 1, 1).await.unwrap();
    assert_eq!(count, 2);

    NotificationService::mark_read(&pool, 1, 1, n2.id).await.unwrap();
    let count2 = NotificationService::unread_count(&pool, 1, 1).await.unwrap();
    assert_eq!(count2, 1);
}

#[tokio::test]
async fn template_rendering() {
    let pool = common::test_pool().await;
    seed_user(&pool).await;
    NotificationService::create_template(
        &pool,
        1,
        &CreateTemplateRequest {
            code: "po_approved".into(),
            title: "采购审批".into(),
            content_template: "订单 {order_no} 金额 {amount} 已批准".into(),
            channel: Some("in_app".into()),
        },
    )
    .await
    .unwrap();

    let n = NotificationService::send_from_template(
        &pool, 1, 1, "po_approved",
        &[("order_no".to_string(), "PO-99".into()), ("amount".to_string(), "5000".into())],
    )
    .await
    .unwrap();
    assert_eq!(n.title, "采购审批");
    assert_eq!(n.content.as_deref(), Some("订单 PO-99 金额 5000 已批准"));
}

#[tokio::test]
async fn preference_upsert() {
    let pool = common::test_pool().await;
    seed_user(&pool).await;
    let pref = NotificationService::update_preference(
        &pool, 1,
        &UpdatePreferenceRequest { notify_type: "finance".into(), channel: Some("email".into()), enabled: false },
    )
    .await
    .unwrap();
    assert!(!pref.enabled);

    // Upsert flips it back.
    let on = NotificationService::update_preference(
        &pool, 1,
        &UpdatePreferenceRequest { notify_type: "finance".into(), channel: Some("email".into()), enabled: true },
    )
    .await
    .unwrap();
    assert!(on.enabled);

    let prefs = NotificationService::list_preferences(&pool, 1).await.unwrap();
    assert_eq!(prefs.len(), 1, "upsert must not duplicate rows");
}
