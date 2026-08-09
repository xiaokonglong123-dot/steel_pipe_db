//! HR row models — sqlx `FromRow` structs mirroring `027_create_hr.sql`.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct HrEmployee {
    pub id: i64,
    pub tenant_id: i64,
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
    pub status: String,
    pub base_salary: Option<f64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct HrPosition {
    pub id: i64,
    pub tenant_id: i64,
    pub department_id: Option<i64>,
    pub title: String,
    pub level: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct HrAttendance {
    pub id: i64,
    pub employee_id: i64,
    pub work_date: NaiveDate,
    pub check_in: Option<DateTime<Utc>>,
    pub check_out: Option<DateTime<Utc>>,
    pub status: String,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct HrAttendanceRule {
    pub id: i64,
    pub tenant_id: i64,
    pub name: String,
    pub department_id: Option<i64>,
    pub work_start_time: chrono::NaiveTime,
    pub work_end_time: chrono::NaiveTime,
    pub grace_minutes: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct HrSalary {
    pub id: i64,
    pub tenant_id: i64,
    pub employee_id: i64,
    pub period: String,
    pub base_salary: f64,
    pub allowance: f64,
    pub commission: f64,
    pub deduction: f64,
    pub social_security: f64,
    pub gross: f64,
    pub net: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct HrContract {
    pub id: i64,
    pub tenant_id: i64,
    pub employee_id: i64,
    pub contract_no: String,
    pub contract_type: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
