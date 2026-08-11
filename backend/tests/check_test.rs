mod common;
#[path = "check_test/support.rs"]
mod support;

use axum::http::StatusCode;
use support::{create_check, fixture, receive};

#[tokio::test]
async fn create_check_session_snapshots_system_qty() {
    let (server, token, item_id, location_id) = fixture().await;
    receive(&server, &token, item_id, location_id).await;

    let (session_id, _) = create_check(&server, &token, location_id).await;
    let (_, check) = server
        .req(
            "GET",
            &format!("/check-records/{session_id}"),
            "",
            Some(&token),
        )
        .await;

    assert_eq!(check["data"]["details"][0]["item_id"], item_id);
    assert_eq!(check["data"]["details"][0]["system_qty"], 100.0);
}

#[tokio::test]
async fn record_actual_qty_computes_diff() {
    let (server, token, item_id, location_id) = fixture().await;
    receive(&server, &token, item_id, location_id).await;
    let (session_id, detail_id) = create_check(&server, &token, location_id).await;

    let body = format!(r#"{{"detail_id":{detail_id},"actual_qty":95}}"#);
    let (status, response) = server
        .req(
            "PUT",
            &format!("/check-records/{session_id}"),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "record actual: {response}");
    let (_, check) = server
        .req(
            "GET",
            &format!("/check-records/{session_id}"),
            "",
            Some(&token),
        )
        .await;
    assert_eq!(check["data"]["details"][0]["actual_qty"], 95.0);
    assert_eq!(check["data"]["details"][0]["diff_qty"], -5.0);
}

#[tokio::test]
async fn post_check_session_updates_inventory() {
    let (server, token, item_id, location_id) = fixture().await;
    receive(&server, &token, item_id, location_id).await;
    let (session_id, detail_id) = create_check(&server, &token, location_id).await;
    let body = format!(r#"{{"detail_id":{detail_id},"actual_qty":95}}"#);
    server
        .req(
            "PUT",
            &format!("/check-records/{session_id}"),
            &body,
            Some(&token),
        )
        .await;

    let (status, posted) = server
        .req(
            "POST",
            &format!("/check-records/{session_id}/post"),
            "",
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "post check: {posted}");
    let quantity: f64 =
        sqlx::query_scalar("SELECT quantity FROM inventory WHERE item_id = ? AND location_id = ?")
            .bind(item_id)
            .bind(location_id)
            .fetch_one(&server.pool)
            .await
            .unwrap();
    let log: (String, f64) = sqlx::query_as(
        "SELECT change_type, quantity FROM inventory_logs
         WHERE ref_type = 'check' AND ref_id = ?",
    )
    .bind(session_id)
    .fetch_one(&server.pool)
    .await
    .unwrap();
    assert_eq!(quantity, 95.0);
    assert_eq!(log, ("check_adjust".to_owned(), -5.0));
}

#[tokio::test]
async fn post_with_no_diff_does_not_change_inventory() {
    let (server, token, item_id, location_id) = fixture().await;
    receive(&server, &token, item_id, location_id).await;
    let (session_id, detail_id) = create_check(&server, &token, location_id).await;
    let body = format!(r#"{{"detail_id":{detail_id},"actual_qty":100}}"#);
    server
        .req(
            "PUT",
            &format!("/check-records/{session_id}"),
            &body,
            Some(&token),
        )
        .await;
    let (status, _) = server
        .req(
            "POST",
            &format!("/check-records/{session_id}/post"),
            "",
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let quantity: f64 =
        sqlx::query_scalar("SELECT quantity FROM inventory WHERE item_id = ? AND location_id = ?")
            .bind(item_id)
            .bind(location_id)
            .fetch_one(&server.pool)
            .await
            .unwrap();
    let logs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inventory_logs WHERE ref_type = 'check' AND ref_id = ?",
    )
    .bind(session_id)
    .fetch_one(&server.pool)
    .await
    .unwrap();
    assert_eq!(quantity, 100.0);
    assert_eq!(logs, 0);
}

#[tokio::test]
async fn cannot_post_draft_without_actual_qty() {
    let (server, token, item_id, location_id) = fixture().await;
    receive(&server, &token, item_id, location_id).await;
    let (session_id, _) = create_check(&server, &token, location_id).await;
    let (status, response) = server
        .req(
            "POST",
            &format!("/check-records/{session_id}/post"),
            "",
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "post draft: {response}");
    assert_eq!(response["code"], 10002);
}
