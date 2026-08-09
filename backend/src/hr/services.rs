//! HR services — employee/position/attendance/salary/contract business logic.

use chrono::NaiveDate;
use sqlx::SqlitePool;

use crate::dto::hr_dto::{
    CheckInRequest, CreateContractRequest, CreateEmployeeRequest, CreatePositionRequest,
    UpdateEmployeeRequest,
};
use crate::error::AppError;
use crate::hr::repos::{
    HrAttendanceRepo, HrAttendanceRuleRepo, HrContractRepo, HrEmployeeRepo, HrPositionRepo,
    HrSalaryRepo,
};
use crate::models::hr::{
    HrAttendance, HrAttendanceRule, HrContract, HrEmployee, HrPosition, HrSalary,
};

pub struct HrService;

impl HrService {
    // -----------------------------------------------------------------------
    // Employees
    // -----------------------------------------------------------------------

    pub async fn list_employees(
        pool: &SqlitePool,
        tenant_id: i64,
        department_id: Option<i64>,
        status: Option<&str>,
        keyword: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<HrEmployee>, i64), AppError> {
        HrEmployeeRepo::list(pool, tenant_id, department_id, status, keyword, page, page_size)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_employee(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<HrEmployee, AppError> {
        HrEmployeeRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Employee not found: {}", id)))
    }

    pub async fn create_employee(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &CreateEmployeeRequest,
    ) -> Result<HrEmployee, AppError> {
        if dto.employee_no.trim().is_empty() {
            return Err(AppError::Validation("Employee number is required".into()));
        }
        if dto.name.trim().is_empty() {
            return Err(AppError::Validation("Employee name is required".into()));
        }
        // Duplicate employee_no guard.
        let dup = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM hr_employees WHERE tenant_id = ? AND employee_no = ? AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(&dto.employee_no)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        if dup > 0 {
            return Err(AppError::Validation(format!(
                "Employee number '{}' already exists",
                dto.employee_no
            )));
        }

        let mut employee = HrEmployee {
            id: 0,
            tenant_id,
            employee_no: dto.employee_no.trim().to_string(),
            user_id: dto.user_id,
            name: dto.name.trim().to_string(),
            gender: dto.gender.clone(),
            birth_date: dto.birth_date,
            id_card: dto.id_card.clone(),
            phone: dto.phone.clone(),
            email: dto.email.clone(),
            department_id: dto.department_id,
            position_id: dto.position_id,
            hire_date: dto.hire_date,
            probation_end: dto.probation_end,
            status: "active".into(),
            base_salary: dto.base_salary,
            notes: dto.notes.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        };
        if employee.probation_end.is_none() {
            // Default probation: 3 months from hire date.
            employee.probation_end = Some(dto.hire_date + chrono::Duration::days(90));
        }
        HrEmployeeRepo::create(pool, &employee)
            .await
            .map_err(AppError::from)
    }

    pub async fn update_employee(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        dto: &UpdateEmployeeRequest,
    ) -> Result<HrEmployee, AppError> {
        // Ensure exists first.
        Self::get_employee(pool, tenant_id, id).await?;
        let mut fields: Vec<(&str, String)> = Vec::new();
        if let Some(v) = &dto.name {
            fields.push(("name", v.clone()));
        }
        if let Some(v) = &dto.gender {
            fields.push(("gender", v.clone()));
        }
        if let Some(v) = &dto.phone {
            fields.push(("phone", v.clone()));
        }
        if let Some(v) = &dto.email {
            fields.push(("email", v.clone()));
        }
        if let Some(v) = dto.department_id {
            fields.push(("department_id", v.to_string()));
        }
        if let Some(v) = dto.position_id {
            fields.push(("position_id", v.to_string()));
        }
        if let Some(v) = dto.probation_end {
            fields.push(("probation_end", v.to_string()));
        }
        if let Some(v) = dto.base_salary {
            fields.push(("base_salary", v.to_string()));
        }
        if let Some(v) = &dto.notes {
            fields.push(("notes", v.clone()));
        }
        if fields.is_empty() {
            return Self::get_employee(pool, tenant_id, id).await;
        }
        HrEmployeeRepo::update(pool, tenant_id, id, &fields)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Employee not found: {}", id)))
    }

    pub async fn terminate_employee(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        reason: Option<&str>,
    ) -> Result<HrEmployee, AppError> {
        let mut fields = vec![("status", "terminated".to_string())];
        if let Some(r) = reason {
            fields.push(("notes", format!("[terminated] {}", r)));
        }
        HrEmployeeRepo::update(pool, tenant_id, id, &fields)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Employee not found: {}", id)))
    }

    pub async fn delete_employee(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<(), AppError> {
        let deleted = HrEmployeeRepo::delete(pool, tenant_id, id)
            .await
            .map_err(AppError::from)?;
        if !deleted {
            return Err(AppError::NotFound(format!("Employee not found: {}", id)));
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Positions
    // -----------------------------------------------------------------------

    pub async fn list_positions(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<HrPosition>, AppError> {
        HrPositionRepo::list(pool, tenant_id).await.map_err(AppError::from)
    }

    pub async fn create_position(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &CreatePositionRequest,
    ) -> Result<HrPosition, AppError> {
        if dto.title.trim().is_empty() {
            return Err(AppError::Validation("Position title is required".into()));
        }
        HrPositionRepo::create(
            pool,
            tenant_id,
            dto.title.trim(),
            dto.department_id,
            dto.level.as_deref(),
            dto.description.as_deref(),
        )
        .await
        .map_err(AppError::from)
    }

    // -----------------------------------------------------------------------
    // Attendance
    // -----------------------------------------------------------------------

    pub async fn list_attendance(
        pool: &SqlitePool,
        employee_id: Option<i64>,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<HrAttendance>, AppError> {
        HrAttendanceRepo::list(pool, employee_id, from, to, 500)
            .await
            .map_err(AppError::from)
    }

    pub async fn check_in(pool: &SqlitePool, dto: &CheckInRequest) -> Result<HrAttendance, AppError> {
        // Employee must exist.
        let _ = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM hr_employees WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(dto.employee_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

        let now = chrono::Utc::now();
        let work_date = now.date_naive();
        HrAttendanceRepo::upsert_check_in(
            pool,
            dto.employee_id,
            work_date,
            dto.check_in.or(Some(now)),
            dto.check_out,
            dto.remark.as_deref(),
        )
        .await
        .map_err(AppError::from)
    }

    pub async fn list_rules(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<HrAttendanceRule>, AppError> {
        HrAttendanceRuleRepo::list(pool, tenant_id).await.map_err(AppError::from)
    }

    // -----------------------------------------------------------------------
    // Salaries
    // -----------------------------------------------------------------------

    pub async fn list_salaries(
        pool: &SqlitePool,
        tenant_id: i64,
        period: Option<&str>,
    ) -> Result<Vec<HrSalary>, AppError> {
        HrSalaryRepo::list(pool, tenant_id, period).await.map_err(AppError::from)
    }

    pub async fn get_salary(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<HrSalary, AppError> {
        HrSalaryRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Salary not found: {}", id)))
    }

    /// Generate payroll for a period: one row per active employee using their
    /// base salary (v1 — allowances/deductions via later phases).
    pub async fn generate_salaries(
        pool: &SqlitePool,
        tenant_id: i64,
        period: &str,
    ) -> Result<Vec<HrSalary>, AppError> {
        let employees = sqlx::query_as::<_, (i64, f64)>(
            "SELECT id, COALESCE(base_salary, 0) FROM hr_employees \
             WHERE tenant_id = ? AND deleted_at IS NULL AND status = 'active'",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        let mut out = Vec::with_capacity(employees.len());
        for (eid, base) in employees {
            let s = HrSalaryRepo::upsert_for_employee(pool, tenant_id, eid, period, base)
                .await
                .map_err(AppError::from)?;
            out.push(s);
        }
        Ok(out)
    }

    // -----------------------------------------------------------------------
    // Contracts
    // -----------------------------------------------------------------------

    pub async fn create_contract(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &CreateContractRequest,
    ) -> Result<HrContract, AppError> {
        if dto.contract_no.trim().is_empty() {
            return Err(AppError::Validation("Contract number is required".into()));
        }
        HrContractRepo::create(
            pool,
            tenant_id,
            dto.employee_id,
            dto.contract_no.trim(),
            &dto.contract_type,
            dto.start_date,
            dto.end_date,
        )
        .await
        .map_err(AppError::from)
    }

    pub async fn list_contracts(pool: &SqlitePool, employee_id: i64) -> Result<Vec<HrContract>, AppError> {
        HrContractRepo::list_for_employee(pool, employee_id)
            .await
            .map_err(AppError::from)
    }
}
