//! HR repositories — pure SQL, static methods, soft-delete aware.

use sqlx::{PgPool, QueryBuilder, Postgres};
use crate::models::hr::{
    HrAttendance, HrAttendanceRule, HrContract, HrEmployee, HrPosition, HrSalary,
};

pub struct HrEmployeeRepo;

impl HrEmployeeRepo {
    #[allow(clippy::too_many_arguments)]
    pub async fn list(
        pool: &PgPool,
        tenant_id: i64,
        department_id: Option<i64>,
        status: Option<&str>,
        keyword: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<HrEmployee>, i64), sqlx::Error> {
        // Count query (plain query — hand-written $N is fine).
        let mut count_sql = String::from(
            "SELECT COUNT(*) FROM hr_employees WHERE tenant_id = $1 AND deleted_at IS NULL",
        );
        let mut binds: Vec<String> = Vec::new();
        let mut n = 2;
        if let Some(dept) = department_id {
            count_sql.push_str(&format!(" AND department_id = ${}", n));
            n += 1;
            binds.push(dept.to_string());
        }
        if let Some(st) = status {
            count_sql.push_str(&format!(" AND status = ${}", n));
            n += 1;
            binds.push(st.to_string());
        }
        if let Some(kw) = keyword {
            count_sql.push_str(&format!(" AND (name ILIKE ${} OR employee_no ILIKE ${})", n, n + 1));
            binds.push(format!("%{}%", kw));
            binds.push(format!("%{}%", kw));
        }
        let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql).bind(tenant_id);
        for v in &binds {
            count_q = count_q.bind(v);
        }
        let total: i64 = count_q.fetch_one(pool).await?;

        // Page query — QueryBuilder's push_bind auto-numbers $1..$N in call
        // order, so the filter is chained push()+push_bind() with NO
        // hand-written placeholders.
        let mut qb = QueryBuilder::<Postgres>::new(
            "SELECT id, tenant_id, employee_no, user_id, name, gender, birth_date, id_card, \
                    phone, email, department_id, position_id, hire_date, probation_end, status, \
                    base_salary, notes, created_at, updated_at, deleted_at \
             FROM hr_employees WHERE tenant_id = ",
        );
        qb.push_bind(tenant_id);
        qb.push(" AND deleted_at IS NULL");
        if let Some(dept) = department_id {
            qb.push(" AND department_id = ");
            qb.push_bind(dept);
        }
        if let Some(st) = status {
            qb.push(" AND status = ");
            qb.push_bind(st);
        }
        if let Some(kw) = keyword {
            qb.push(" AND (name ILIKE ");
            qb.push_bind(format!("%{}%", kw));
            qb.push(" OR employee_no ILIKE ");
            qb.push_bind(format!("%{}%", kw));
            qb.push(")");
        }
        qb.push(" ORDER BY id DESC LIMIT ");
        qb.push_bind(page_size);
        qb.push(" OFFSET ");
        qb.push_bind((page - 1) * page_size);
        let items = qb.build_query_as::<HrEmployee>().fetch_all(pool).await?;
        Ok((items, total))
    }

    pub async fn find_by_id(pool: &PgPool, tenant_id: i64, id: i64) -> Result<Option<HrEmployee>, sqlx::Error> {
        sqlx::query_as::<_, HrEmployee>(
            "SELECT id, tenant_id, employee_no, user_id, name, gender, birth_date, id_card, \
                    phone, email, department_id, position_id, hire_date, probation_end, status, \
                    base_salary, notes, created_at, updated_at, deleted_at \
             FROM hr_employees WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &PgPool, e: &HrEmployee) -> Result<HrEmployee, sqlx::Error> {
        sqlx::query_as::<_, HrEmployee>(
            "INSERT INTO hr_employees \
             (tenant_id, employee_no, user_id, name, gender, birth_date, id_card, phone, email, \
              department_id, position_id, hire_date, probation_end, status, base_salary, notes) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16) \
             RETURNING id, tenant_id, employee_no, user_id, name, gender, birth_date, id_card, \
                       phone, email, department_id, position_id, hire_date, probation_end, status, \
                       base_salary, notes, created_at, updated_at, deleted_at",
        )
        .bind(e.tenant_id)
        .bind(&e.employee_no)
        .bind(e.user_id)
        .bind(&e.name)
        .bind(&e.gender)
        .bind(e.birth_date)
        .bind(&e.id_card)
        .bind(&e.phone)
        .bind(&e.email)
        .bind(e.department_id)
        .bind(e.position_id)
        .bind(e.hire_date)
        .bind(e.probation_end)
        .bind(&e.status)
        .bind(e.base_salary)
        .bind(&e.notes)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
        fields: &[(&str, String)],
    ) -> Result<Option<HrEmployee>, sqlx::Error> {
        // Typed UPDATE: numeric/date columns must NOT be bound as text
        // (PG rejects text → bigint/date). Each supported column is
        // explicitly cast from the text payload.
        let mut qb = QueryBuilder::<Postgres>::new("UPDATE hr_employees SET ");
        let mut first = true;
        for (col, val) in fields {
            if !first {
                qb.push(", ");
            }
            match *col {
                "name" | "gender" | "phone" | "email" | "notes" => {
                    qb.push(col);
                    qb.push(" = ");
                    qb.push_bind(val);
                }
                "department_id" | "position_id" => {
                    qb.push(col);
                    qb.push(" = ");
                    qb.push_bind(val.as_str().parse::<i64>().unwrap_or(0));
                }
                "probation_end" | "hire_date" => {
                    qb.push(col);
                    qb.push(" = ");
                    qb.push_bind(val.as_str().parse::<chrono::NaiveDate>().ok());
                }
                "base_salary" => {
                    qb.push(col);
                    qb.push(" = ");
                    qb.push_bind(val.as_str().parse::<rust_decimal::Decimal>().unwrap_or_default());
                }
                _ => {
                    qb.push(col);
                    qb.push(" = ");
                    qb.push_bind(val);
                }
            }
            first = false;
        }
        qb.push(", updated_at = NOW() WHERE tenant_id = ");
        qb.push_bind(tenant_id);
        qb.push(" AND id = ");
        qb.push_bind(id);
        qb.push(" AND deleted_at IS NULL RETURNING id, tenant_id, employee_no, user_id, name, gender, birth_date, id_card, phone, email, department_id, position_id, hire_date, probation_end, status, base_salary, notes, created_at, updated_at, deleted_at");
        qb.build_query_as::<HrEmployee>().fetch_optional(pool).await
    }

    pub async fn delete(pool: &PgPool, tenant_id: i64, id: i64) -> Result<bool, sqlx::Error> {
        sqlx::query("UPDATE hr_employees SET deleted_at = NOW() WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL")
            .bind(tenant_id)
            .bind(id)
            .execute(pool)
            .await
            .map(|r| r.rows_affected() > 0)
    }
}

pub struct HrPositionRepo;

impl HrPositionRepo {
    pub async fn list(pool: &PgPool, tenant_id: i64) -> Result<Vec<HrPosition>, sqlx::Error> {
        sqlx::query_as::<_, HrPosition>(
            "SELECT id, tenant_id, department_id, title, level, description, is_active, \
                    created_at, updated_at, deleted_at \
             FROM hr_positions WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY title",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &PgPool, tenant_id: i64, title: &str, department_id: Option<i64>, level: Option<&str>, description: Option<&str>) -> Result<HrPosition, sqlx::Error> {
        sqlx::query_as::<_, HrPosition>(
            "INSERT INTO hr_positions (tenant_id, department_id, title, level, description) \
             VALUES ($1, $2, $3, $4, $5) \
             RETURNING id, tenant_id, department_id, title, level, description, is_active, \
                       created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(department_id)
        .bind(title)
        .bind(level)
        .bind(description)
        .fetch_one(pool)
        .await
    }
}

pub struct HrAttendanceRepo;

impl HrAttendanceRepo {
    pub async fn list(
        pool: &PgPool,
        employee_id: Option<i64>,
        from: Option<chrono::NaiveDate>,
        to: Option<chrono::NaiveDate>,
        limit: i64,
    ) -> Result<Vec<HrAttendance>, sqlx::Error> {
        // QueryBuilder's push_bind auto-numbers $1..$N in call order, so we
        // must NOT hand-write placeholders — chain push()+push_bind() instead.
        let mut qb = QueryBuilder::<Postgres>::new(
            "SELECT id, employee_id, work_date, check_in, check_out, status, remark, created_at \
             FROM hr_attendances WHERE ",
        );
        let mut first = true;
        if let Some(eid) = employee_id {
            qb.push("employee_id = ");
            qb.push_bind(eid);
            first = false;
        }
        if let Some(f) = from {
            if !first {
                qb.push(" AND ");
            }
            qb.push("work_date >= ");
            qb.push_bind(f);
            first = false;
        }
        if let Some(t) = to {
            if !first {
                qb.push(" AND ");
            }
            qb.push("work_date <= ");
            qb.push_bind(t);
            first = false;
        }
        if first {
            qb.push("TRUE");
        }
        qb.push(" ORDER BY work_date DESC LIMIT ");
        qb.push_bind(limit);
        qb.build_query_as::<HrAttendance>().fetch_all(pool).await
    }

    pub async fn upsert_check_in(
        pool: &PgPool,
        employee_id: i64,
        work_date: chrono::NaiveDate,
        check_in: Option<chrono::DateTime<chrono::Utc>>,
        check_out: Option<chrono::DateTime<chrono::Utc>>,
        remark: Option<&str>,
    ) -> Result<HrAttendance, sqlx::Error> {
        sqlx::query_as::<_, HrAttendance>(
            "INSERT INTO hr_attendances (employee_id, work_date, check_in, check_out, remark) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (employee_id, work_date) DO UPDATE SET \
               check_in = COALESCE(EXCLUDED.check_in, hr_attendances.check_in), \
               check_out = COALESCE(EXCLUDED.check_out, hr_attendances.check_out), \
               remark = COALESCE(EXCLUDED.remark, hr_attendances.remark) \
             RETURNING id, employee_id, work_date, check_in, check_out, status, remark, created_at",
        )
        .bind(employee_id)
        .bind(work_date)
        .bind(check_in)
        .bind(check_out)
        .bind(remark)
        .fetch_one(pool)
        .await
    }
}

pub struct HrSalaryRepo;

impl HrSalaryRepo {
    pub async fn list(pool: &PgPool, tenant_id: i64, period: Option<&str>) -> Result<Vec<HrSalary>, sqlx::Error> {
        sqlx::query_as::<_, HrSalary>(
            "SELECT id, tenant_id, employee_id, period, base_salary, allowance, commission, \
                    deduction, social_security, gross, net, status, created_at \
             FROM hr_salaries WHERE tenant_id = $1 AND ($2::text IS NULL OR period = $2) \
             ORDER BY period DESC, id",
        )
        .bind(tenant_id)
        .bind(period)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, tenant_id: i64, id: i64) -> Result<Option<HrSalary>, sqlx::Error> {
        sqlx::query_as::<_, HrSalary>(
            "SELECT id, tenant_id, employee_id, period, base_salary, allowance, commission, \
                    deduction, social_security, gross, net, status, created_at \
             FROM hr_salaries WHERE tenant_id = $1 AND id = $2",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Generate a payroll row for one employee for a period from their
    /// base_salary (v1: no allowances/deductions beyond base).
    pub async fn upsert_for_employee(
        pool: &PgPool,
        tenant_id: i64,
        employee_id: i64,
        period: &str,
        base_salary: rust_decimal::Decimal,
    ) -> Result<HrSalary, sqlx::Error> {
        sqlx::query_as::<_, HrSalary>(
            "INSERT INTO hr_salaries \
             (tenant_id, employee_id, period, base_salary, allowance, commission, deduction, \
              social_security, gross, net) \
             VALUES ($1, $2, $3, $4, 0, 0, 0, 0, $4, $4) \
             ON CONFLICT (employee_id, period) DO UPDATE SET \
               base_salary = EXCLUDED.base_salary, gross = EXCLUDED.gross, net = EXCLUDED.net \
             RETURNING id, tenant_id, employee_id, period, base_salary, allowance, commission, \
                       deduction, social_security, gross, net, status, created_at",
        )
        .bind(tenant_id)
        .bind(employee_id)
        .bind(period)
        .bind(base_salary)
        .fetch_one(pool)
        .await
    }
}

pub struct HrContractRepo;

impl HrContractRepo {
    pub async fn create(
        pool: &PgPool,
        tenant_id: i64,
        employee_id: i64,
        contract_no: &str,
        contract_type: &str,
        start_date: chrono::NaiveDate,
        end_date: Option<chrono::NaiveDate>,
    ) -> Result<HrContract, sqlx::Error> {
        sqlx::query_as::<_, HrContract>(
            "INSERT INTO hr_contracts \
             (tenant_id, employee_id, contract_no, contract_type, start_date, end_date) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, tenant_id, employee_id, contract_no, contract_type, start_date, \
                       end_date, status, created_at, updated_at",
        )
        .bind(tenant_id)
        .bind(employee_id)
        .bind(contract_no)
        .bind(contract_type)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await
    }

    pub async fn list_for_employee(pool: &PgPool, employee_id: i64) -> Result<Vec<HrContract>, sqlx::Error> {
        sqlx::query_as::<_, HrContract>(
            "SELECT id, tenant_id, employee_id, contract_no, contract_type, start_date, \
                    end_date, status, created_at, updated_at \
             FROM hr_contracts WHERE employee_id = $1 ORDER BY start_date DESC",
        )
        .bind(employee_id)
        .fetch_all(pool)
        .await
    }
}

pub struct HrAttendanceRuleRepo;

impl HrAttendanceRuleRepo {
    pub async fn list(pool: &PgPool, tenant_id: i64) -> Result<Vec<HrAttendanceRule>, sqlx::Error> {
        sqlx::query_as::<_, HrAttendanceRule>(
            "SELECT id, tenant_id, name, department_id, work_start_time, work_end_time, \
                    grace_minutes, is_active, created_at \
             FROM hr_attendance_rules WHERE tenant_id = $1 AND is_active = TRUE ORDER BY id",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }
}
