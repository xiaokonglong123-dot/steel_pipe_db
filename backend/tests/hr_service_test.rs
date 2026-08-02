//! HR integration tests — employees, positions, attendance, salaries, contracts.

mod common;

use steel_pipe_db::dto::hr_dto::{
    CheckInRequest, CreateContractRequest, CreateEmployeeRequest, CreatePositionRequest,
    UpdateEmployeeRequest,
};
use steel_pipe_db::hr::services::HrService;

fn employee_dto(no: &str, name: &str) -> CreateEmployeeRequest {
    CreateEmployeeRequest {
        employee_no: no.into(),
        user_id: None,
        name: name.into(),
        gender: Some("M".into()),
        birth_date: None,
        id_card: None,
        phone: Some("13800000000".into()),
        email: None,
        department_id: None,
        position_id: None,
        hire_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        probation_end: None,
        base_salary: Some(rust_decimal_macros::dec!(8000)),
        notes: None,
    }
}

#[tokio::test]
async fn create_and_get_employee() {
    let pool = common::test_pool().await;
    let e = HrService::create_employee(&pool, 1, &employee_dto("E001", "张三")).await.unwrap();
    assert_eq!(e.status, "active");
    // Default probation = 3 months after hire.
    assert_eq!(e.probation_end.unwrap(), chrono::NaiveDate::from_ymd_opt(2026, 4, 15).unwrap());

    let got = HrService::get_employee(&pool, 1, e.id).await.unwrap();
    assert_eq!(got.name, "张三");
    assert_eq!(got.employee_no, "E001");
}

#[tokio::test]
async fn duplicate_employee_no_rejected() {
    let pool = common::test_pool().await;
    HrService::create_employee(&pool, 1, &employee_dto("E001", "张三")).await.unwrap();
    let dup = HrService::create_employee(&pool, 1, &employee_dto("E001", "李四")).await;
    assert!(dup.is_err(), "duplicate employee_no must be rejected");
}

#[tokio::test]
async fn update_and_terminate_employee() {
    let pool = common::test_pool().await;
    let e = HrService::create_employee(&pool, 1, &employee_dto("E002", "王五")).await.unwrap();

    let upd = UpdateEmployeeRequest {
        name: Some("王五改".into()),
        gender: None,
        phone: Some("13911112222".into()),
        email: None,
        department_id: Some(1),
        position_id: None,
        probation_end: None,
        base_salary: Some(rust_decimal_macros::dec!(9000)),
        notes: None,
    };
    let updated = HrService::update_employee(&pool, 1, e.id, &upd).await.unwrap();
    assert_eq!(updated.name, "王五改");
    assert_eq!(updated.department_id, Some(1));

    let terminated = HrService::terminate_employee(&pool, 1, e.id, Some("个人原因")).await.unwrap();
    assert_eq!(terminated.status, "terminated");
}

#[tokio::test]
async fn list_employees_pagination_and_filter() {
    let pool = common::test_pool().await;
    HrService::create_employee(&pool, 1, &employee_dto("E010", "赵六")).await.unwrap();
    HrService::create_employee(&pool, 1, &employee_dto("E011", "钱七")).await.unwrap();

    let (items, total) = HrService::list_employees(&pool, 1, None, None, Some("赵"), 1, 20).await.unwrap();
    assert_eq!(total, 1);
    assert_eq!(items[0].name, "赵六");

    let (_, total_all) = HrService::list_employees(&pool, 1, None, None, None, 1, 20).await.unwrap();
    assert_eq!(total_all, 2);
}

#[tokio::test]
async fn position_crud() {
    let pool = common::test_pool().await;
    let p = HrService::create_position(
        &pool,
        1,
        &CreatePositionRequest {
            title: "车间主任".into(),
            department_id: None,
            level: Some("M2".into()),
            description: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(p.title, "车间主任");
    let list = HrService::list_positions(&pool, 1).await.unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn check_in_and_list_attendance() {
    let pool = common::test_pool().await;
    let e = HrService::create_employee(&pool, 1, &employee_dto("E020", "孙八")).await.unwrap();
    let att = HrService::check_in(
        &pool,
        &CheckInRequest {
            employee_id: e.id,
            check_in: None,
            check_out: None,
            remark: Some("正常".into()),
        },
    )
    .await
    .unwrap();
    assert_eq!(att.status, "normal");
    assert!(att.check_in.is_some());

    let list = HrService::list_attendance(&pool, Some(e.id), None, None).await.unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn generate_and_list_salaries() {
    let pool = common::test_pool().await;
    let e = HrService::create_employee(&pool, 1, &employee_dto("E030", "周九")).await.unwrap();
    let salaries = HrService::generate_salaries(&pool, 1, "2026-07").await.unwrap();
    assert_eq!(salaries.len(), 1);
    assert_eq!(salaries[0].employee_id, e.id);
    assert_eq!(salaries[0].gross, rust_decimal_macros::dec!(8000));
    assert_eq!(salaries[0].net, rust_decimal_macros::dec!(8000));

    // Re-generate is idempotent (upsert).
    let again = HrService::generate_salaries(&pool, 1, "2026-07").await.unwrap();
    assert_eq!(again.len(), 1);
}

#[tokio::test]
async fn contract_lifecycle() {
    let pool = common::test_pool().await;
    let e = HrService::create_employee(&pool, 1, &employee_dto("E040", "吴十")).await.unwrap();
    let c = HrService::create_contract(
        &pool,
        1,
        &CreateContractRequest {
            employee_id: e.id,
            contract_no: "HT-2026-001".into(),
            contract_type: "fixed".into(),
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            end_date: Some(chrono::NaiveDate::from_ymd_opt(2028, 1, 14).unwrap()),
        },
    )
    .await
    .unwrap();
    assert_eq!(c.status, "active");
    let list = HrService::list_contracts(&pool, e.id).await.unwrap();
    assert_eq!(list.len(), 1);
}
