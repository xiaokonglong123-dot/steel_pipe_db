//! Project management integration tests — projects, WBS, budget.

mod common;

use erp_server::dto::project_dto::{
    CreateProjectRequest, CreateTransactionRequest, CreateWbsRequest, UpdateWbsProgressRequest,
};
use erp_server::project::services::ProjectService;

#[tokio::test]
async fn project_lifecycle() {
    let pool = common::test_pool().await;
    let project = ProjectService::create_project(
        &pool,
        1,
        &CreateProjectRequest {
            name: "一号产线扩建项目".into(),
            description: Some("一期".into()),
            start_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()),
            end_date: None,
            manager_id: None,
            budget: Some(500000.0),
        },
    )
    .await
    .unwrap();
    assert_eq!(project.status, "planning");
    assert_eq!(project.budget, 500000.0);

    let active = ProjectService::update_project_status(&pool, 1, project.id, "active").await.unwrap();
    assert_eq!(active.status, "active");
}

#[tokio::test]
async fn wbs_tree_and_progress() {
    let pool = common::test_pool().await;
    let project = ProjectService::create_project(&pool, 1, &CreateProjectRequest { name: "P2".into(), description: None, start_date: None, end_date: None, manager_id: None, budget: None }).await.unwrap();

    let parent = ProjectService::create_wbs(
        &pool, 1, project.id,
        &CreateWbsRequest { parent_id: None, code: "1".into(), name: "土建工程".into(), weight_pct: Some(60.0), start_date: None, end_date: None, assignee_id: None },
    )
    .await
    .unwrap();
    let child = ProjectService::create_wbs(
        &pool, 1, project.id,
        &CreateWbsRequest { parent_id: Some(parent.id), code: "1.1".into(), name: "基础施工".into(), weight_pct: Some(40.0), start_date: None, end_date: None, assignee_id: None },
    )
    .await
    .unwrap();
    assert_eq!(child.parent_id, Some(parent.id));

    let updated = ProjectService::update_wbs_progress(
        &pool, project.id, child.id,
        &UpdateWbsProgressRequest { progress_pct: 50.0 },
    )
    .await
    .unwrap();
    assert_eq!(updated.progress_pct, 50.0);

    // Invalid progress rejected.
    let err = ProjectService::update_wbs_progress(
        &pool, project.id, child.id,
        &UpdateWbsProgressRequest { progress_pct: 150.0 },
    )
    .await;
    assert!(err.is_err(), "progress over 100 must be rejected");

    let tree = ProjectService::wbs_tree(&pool, project.id).await.unwrap();
    assert_eq!(tree.len(), 2);
}

#[tokio::test]
async fn financials_budget_vs_expense() {
    let pool = common::test_pool().await;
    let project = ProjectService::create_project(&pool, 1, &CreateProjectRequest { name: "P3".into(), description: None, start_date: None, end_date: None, manager_id: None, budget: Some(100000.0) }).await.unwrap();

    ProjectService::create_transaction(
        &pool, 1, project.id,
        &CreateTransactionRequest { tx_type: "expense".into(), amount: 30000.0, description: Some("设备采购".into()), tx_date: None },
        Some(1),
    )
    .await
    .unwrap();
    ProjectService::create_transaction(
        &pool, 1, project.id,
        &CreateTransactionRequest { tx_type: "revenue".into(), amount: 20000.0, description: Some("进度款".into()), tx_date: None },
        Some(1),
    )
    .await
    .unwrap();

    let fin = ProjectService::financials(&pool, 1, project.id).await.unwrap();
    assert_eq!(fin.budget, 100000.0);
    assert_eq!(fin.expense_total, 30000.0);
    assert_eq!(fin.revenue_total, 20000.0);
    assert_eq!(fin.remaining, 70000.0, "budget minus expenses");
}

#[tokio::test]
async fn invalid_transaction_type_rejected() {
    let pool = common::test_pool().await;
    let project = ProjectService::create_project(&pool, 1, &CreateProjectRequest { name: "P4".into(), description: None, start_date: None, end_date: None, manager_id: None, budget: None }).await.unwrap();
    let err = ProjectService::create_transaction(
        &pool, 1, project.id,
        &CreateTransactionRequest { tx_type: "gift".into(), amount: 1.0, description: None, tx_date: None },
        None,
    )
    .await;
    assert!(err.is_err(), "invalid tx_type must be rejected");
}
