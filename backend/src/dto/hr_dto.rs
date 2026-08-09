//! HR DTOs — request payloads for employee/attendance/salary endpoints.

use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateEmployeeRequest {
    pub employee_no: String,
    pub user_id: Option<i64>,
    pub name: String,
    pub gender: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub id_card: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub department_id: Option<i64>,
    pub position_id: Option<i64>,
    pub hire_date: NaiveDate,
    pub probation_end: Option<NaiveDate>,
    pub base_salary: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmployeeRequest {
    pub name: Option<String>,
    pub gender: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub department_id: Option<i64>,
    pub position_id: Option<i64>,
    pub probation_end: Option<NaiveDate>,
    pub base_salary: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TerminateEmployeeRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePositionRequest {
    pub title: String,
    pub department_id: Option<i64>,
    pub level: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckInRequest {
    pub employee_id: i64,
    pub check_in: Option<chrono::DateTime<chrono::Utc>>,
    pub check_out: Option<chrono::DateTime<chrono::Utc>>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AttendanceQuery {
    pub employee_id: Option<i64>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateSalaryRequest {
    pub period: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateContractRequest {
    pub employee_id: i64,
    pub contract_no: String,
    pub contract_type: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}
