//! Project management integration tests — projects, WBS, budget.

mod common;

use rust_decimal_macros::dec;
use steel_pipe_db::dto::project_dto::{
    CreateProjectRequest, CreateTransactionRequest, CreateWbsRequest, UpdateWbsProgressRequest,
};
use steel_pipe_db::project::services::ProjectService;

#[tokio::test]
async fn project_lifecycle() {
    let pool = common::test_pool().await;
    let project = ProjectService::create_project(
        &pool,
        1,
        &CreateProjectRequest {
            name: "油气田开发项目".into(),
            description: Some("一期".into()),
            start_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()),
            end_date: None,
            manager_id: None,
            budget: Some(dec!(500000)),
        },
    )
    .await
    .unwrap();
    assert_eq!(project.status, "planning");
    assert_eq!(project.budget, dec!(500000));

    let active = ProjectService::update_project_status(&pool, 1, project.id, "active").await.unwrap();
    assert_eq!(active.status, "active");
}

#[tokio::test]
async fn wbs_tree_and_progress() {
    let pool = common::test_pool().await;
    let project = ProjectService::create_project(&pool, 1, &CreateProjectRequest { name: "P2".into(), description: None, start_date: None, end_date: None, manager_id: None, budget: None }).await.unwrap();

    let parent = ProjectService::create_wbs(
        &pool, 1, project.id,
        &CreateWbsRequest { parent_id: None, code: "1".into(), name: "钻井".into(), weight_pct: Some(dec!(60)), start_date: None, end_date: None, assignee_id: None },
    )
    .await
    .unwrap();
    let child = ProjectService::create_wbs(
        &pool, 1, project.id,
        &CreateWbsRequest { parent_id: Some(parent.id), code: "1.1".into(), name: "表层钻井".into(), weight_pct: Some(dec!(40)), start_date: None, end_date: None, assignee_id: None },
    )
    .await
    .unwrap();
    assert_eq!(child.parent_id, Some(parent.id));

    let updated = ProjectService::update_wbs_progress(
        &pool, project.id, child.id,
        &UpdateWbsProgressRequest { progress_pct: dec!(50) },
    )
    .await
    .unwrap();
    assert_eq!(updated.progress_pct, dec!(50));

    // Invalid progress rejected.
    let err = ProjectService::update_wbs_progress(
        &pool, project.id, child.id,
        &UpdateWbsProgressRequest { progress_pct: dec!(150) },
    )
    .await;
    assert!(err.is_err(), "progress over 100 must be rejected");

    let tree = ProjectService::wbs_tree(&pool, project.id).await.unwrap();
    assert_eq!(tree.len(), 2);
}

#[tokio::test]
async fn financials_budget_vs_expense() {
    let pool = common::test_pool().await;
    let project = ProjectService::create_project(&pool, 1, &CreateProjectRequest { name: "P3".into(), description: None, start_date: None, end_date: None, manager_id: None, budget: Some(dec!(100000)) }).await.unwrap();

    ProjectService::create_transaction(
        &pool, 1, project.id,
        &CreateTransactionRequest { tx_type: "expense".into(), amount: dec!(30000), description: Some("钢材采购".into()), tx_date: None },
        Some(1),
    )
    .await
    .unwrap();
    ProjectService::create_transaction(
        &pool, 1, project.id,
        &CreateTransactionRequest { tx_type: "revenue".into(), amount: dec!(20000), description: Some("进度款".into()), tx_date: None },
        Some(1),
    )
    .await
    .unwrap();

    let fin = ProjectService::financials(&pool, 1, project.id).await.unwrap();
    assert_eq!(fin.budget, dec!(100000));
    assert_eq!(fin.expense_total, dec!(30000));
    assert_eq!(fin.revenue_total, dec!(20000));
    assert_eq!(fin.remaining, dec!(70000), "budget minus expenses");
}

#[tokio::test]
async fn invalid_transaction_type_rejected() {
    let pool = common::test_pool().await;
    let project = ProjectService::create_project(&pool, 1, &CreateProjectRequest { name: "P4".into(), description: None, start_date: None, end_date: None, manager_id: None, budget: None }).await.unwrap();
    let err = ProjectService::create_transaction(
        &pool, 1, project.id,
        &CreateTransactionRequest { tx_type: "gift".into(), amount: dec!(1), description: None, tx_date: None },
        None,
    )
    .await;
    assert!(err.is_err(), "invalid tx_type must be rejected");
}
