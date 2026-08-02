# 003 — HR & 员工管理 (Phase 1)

> **版本**: v1.0
> **日期**: 2026-08-02
> **依赖**: 001-auth-identity (users, departments)
> **父文档**: 015-architecture-overview

---

## 1. 目标

提供完整的人力资源管理能力：员工档案、部门职责、考勤、薪酬。

## 2. 功能清单

| 模块 | 功能 | 优先级 |
|------|------|--------|
| 员工管理 | 员工档案 (CRUD)、状态生命周期 (在职/离职/停职) | P0 |
| 部门管理 | 层级制部门 + 职务定义 | P0 |
| 入职管理 | 入职时间、试用期 (tracker) | P0 |
| 考勤管理 | 打卡规则、出勤记录、请假记录 | P1 |
| 薪酬管理 | 基本工资 + 津贴 + 扣款 + 提成 + 社保 | P1 |
| 劳动合同 | 合同存储、合同到期提醒 | P1 |
| 绩效管理 | 考核模板 + 评分 (每个员工) | P2 |

## 3. 数据模型

```sql
-- HR schema全部在 new `hr` schema 中

CREATE TABLE hr.employees (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES auth.users(id),
    person_id BIGINT,
    company_id BIGINT NOT NULL,
    department_id BIGINT,
    position_id BIGINT,
    employee_no VARCHAR(50) UNIQUE NOT NULL,
    hire_date DATE NOT NULL,
    termination_date DATE,                  -- 离职日期 (预防)
    employment_type VARCHAR(30),             -- formal / intern / part_time
    status VARCHAR(20) DEFAULT 'active',     -- active / leave / terminated
    manager_id BIGINT REFERENCES hr.employees(id),  -- (直属领导)
    probation_end DATE,                      -- 试用结束
    created_at, updated_at, deleted_at
);

CREATE TABLE hr.departments (   -- extends auth.departments
    id BIGSERIAL PRIMARY KEY,
    auth_dept_id BIGINT,             -- link to auth schema
    code VARCHAR(20),
    company_id BIGINT,
    manager_id BIGINT REFERENCES hr.employees(id),
    budget_limit NUMERIC(18,2) DEFAULT 0
);

CREATE TABLE hr.positions (
    id BIGSERIAL PRIMARY KEY,
    auth_position_id BIGINT,
    department_id BIGINT REFERENCES hr.departments(id),
    title VARCHAR(200),
    job_description TEXT,
    salary_range_low NUMERIC(18,2),
    salary_range_high NUMERIC(18,2)
);

-- 考勤
CREATE TABLE hr.attendances (
    id BIGSERIAL PRIMARY KEY,
    employee_id BIGINT REFERENCES hr.employees(id),
    date DATE NOT NULL,
    clock_in TIMESTAMPTZ,
    clock_out TIMESTAMPTZ,
    status VARCHAR(20) DEFAULT 'normal',    -- normal / late / early / absent / leave
    adjustment_reason TEXT                 -- 也可接受放假变动
);

CREATE TABLE hr.attendance_rules (
    id BIGSERIAL PRIMARY KEY,
    department_id BIGINT,
    work_start_time TIME NOT NULL,                                    -- 上班时间
    work_end_time TIME NOT NULL,
    late_before_minutes INT DEFAULT 10,                               - 迟到
    overtime_penalty BOOLEAN default false
);

-- 薪酬
CREATE TABLE hr.salaries (
    id BIGSERIAL PRIMARY KEY,
    employee_id BIGINT REFERENCES hr.employees(id),
    month DATE NOT NULL,
    gross_salary NUMERIC(18,2) NOT NULL,    (总工资)
    net_salary NUMERIC(18,2),                -- (税后)
    pay_date DATE,
    status VARCHAR(20) DEFAULT 'pending',    -- pending / approved / done
    created_at TIMESTAMPTZ
);

CREATE TABLE hr.salary_items (
    id BIGSERIAL PRIMARY KEY,
    salary_id BIGINT REFERENCES hr.salaries(id),
    item_type VARCHAR(50),                     -- base / overtime / allowance / deduction / bonus / tax
    amount NUMERIC(18,2),
    description TEXT
);

-- 合同
CREATE TABLE hr.contracts (
    id BIGSERIAL PRIMARY KEY,
    employee_id BIGINT REFERENCES hr.employees(id),
    contract_no VARCHAR(100),
    start_date DATE NOT NULL,
    end_date DATE,
    contract_type VARCHAR(50),  -- permanent, fixed_term, probation
    signed_at DATE,
    file_url TEXT,
    created_at, updated_at
);
```

## 4. API

| Method | Path | Description |
|--------|------|-------------|
| 员工 | | |
| GET | `/api/hr/employees` | 列表 (分页 + 过滤) |
| POST | `POST /api/hr/employees` | 创建员工 |
| GET | `/api/hr/employees/:id` | 详情 |
| PUT | `/api/hr/employees/:id` | 更新 |
| POST | `/api/hr/employees/:id/terminate` | 离职处理 |
| 部门 | | |
| GET | `/api/hr/departments` | 部门树 |
| POST | `/api/hr/departments` | 创建部门 |
| 考勤 | | |
| GET | `/api/hr/attendance` | 出勤查询 |
| POST | `/api/hr/attendance/check-in` | 上班打卡参数 (也可以从 broker 同时) |
| 薪酬 | | |
| GET | `/api/hr/salaries` | 薪资列表 |
| POST | `/api/hr/salaries` | 生成工资 |
| GET | `/api/hr/salaries/:id` | 明细 |

## 5. 前后端对应

**后端** (crates/hr/):
- `employees.rs`, `departments.rs`, `attendance.rs`, `salaries.rs`

**前端**:
- `features/hr/pages/EmployeeListPage.tsx`
- `features/hr/pages/AttendancePage.tsx`
- `features/hr/pages/SalaryPage.tsx`

## 6. 与其他模块的联动

- 当员工新建时 → `hr.employee.created` → auth 模块创建系统用户
- 当薪资支付时 → `hr.salary.paid` → finance 创建 journal entry