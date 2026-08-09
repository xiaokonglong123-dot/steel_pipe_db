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
| ------ | ------ | -------- |
| 员工管理 | 员工档案 (CRUD)、状态生命周期 (在职/离职/停职) | P0 |
| 部门管理 | 层级制部门 + 职务定义 | P0 |
| 入职管理 | 入职时间、试用期 (tracker) | P0 |
| 考勤管理 | 打卡规则、出勤记录、请假记录 | P1 |
| 薪酬管理 | 基本工资 + 津贴 + 扣款 + 提成 + 社保 | P1 |
| 劳动合同 | 合同存储、合同到期提醒 | P1 |
| 绩效管理 | 考核模板 + 评分 (每个员工) | P2 |

## 3. 数据模型

> 数据库为 SQLite3（`sqlite://data/erp.db?mode=rwc`），表名不带 schema 前缀，金额用 REAL。

```sql
CREATE TABLE employees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id),
    company_id INTEGER NOT NULL,
    department_id INTEGER,
    position_id INTEGER,
    employee_no TEXT UNIQUE NOT NULL,
    hire_date TEXT NOT NULL,
    termination_date TEXT,                  -- 离职日期
    employment_type TEXT,                   -- formal / intern / part_time
    status TEXT DEFAULT 'active',           -- active / leave / terminated
    manager_id INTEGER REFERENCES employees(id),  -- 直属领导
    probation_end TEXT,                     -- 试用结束
    created_at, updated_at, deleted_at
);

CREATE TABLE departments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id INTEGER,
    code TEXT,
    company_id INTEGER,
    manager_id INTEGER REFERENCES employees(id),
    budget_limit REAL DEFAULT 0
);

CREATE TABLE positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    department_id INTEGER REFERENCES departments(id),
    title TEXT,
    job_description TEXT,
    salary_range_low REAL,
    salary_range_high REAL
);

-- 考勤
CREATE TABLE attendances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER REFERENCES employees(id),
    date TEXT NOT NULL,
    clock_in TEXT,
    clock_out TEXT,
    status TEXT DEFAULT 'normal',   -- normal / late / early / absent / leave
    adjustment_reason TEXT
);

CREATE TABLE attendance_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    department_id INTEGER,
    work_start_time TEXT NOT NULL,
    work_end_time TEXT NOT NULL,
    late_before_minutes INTEGER DEFAULT 10,
    overtime_penalty INTEGER DEFAULT 0
);

-- 薪酬
CREATE TABLE salaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER REFERENCES employees(id),
    month TEXT NOT NULL,
    gross_salary REAL NOT NULL,
    net_salary REAL,
    pay_date TEXT,
    status TEXT DEFAULT 'pending',   -- pending / approved / done
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE salary_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    salary_id INTEGER REFERENCES salaries(id),
    item_type TEXT,                  -- base / overtime / allowance / deduction / bonus / tax
    amount REAL,
    description TEXT
);

-- 合同
CREATE TABLE hr_contracts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER REFERENCES employees(id),
    contract_no TEXT,
    start_date TEXT NOT NULL,
    end_date TEXT,
    contract_type TEXT,  -- permanent, fixed_term, probation
    signed_at TEXT,
    file_url TEXT,
    created_at, updated_at
);
```

## 4. API

| Method | Path | Description |
| -------- | ------ | ------------- |
| 员工 | | |
| GET | `/api/hr/employees` | 列表 (分页 + 过滤) |
| POST | `/api/hr/employees` | 创建员工 |
| GET | `/api/hr/employees/:id` | 详情 |
| PUT | `/api/hr/employees/:id` | 更新 |
| POST | `/api/hr/employees/:id/terminate` | 离职处理 |
| 部门 | | |
| GET | `/api/hr/departments` | 部门树 |
| POST | `/api/hr/departments` | 创建部门 |
| 考勤 | | |
| GET | `/api/hr/attendance` | 出勤查询 |
| POST | `/api/hr/attendance/check-in` | 上班打卡 |
| 薪酬 | | |
| GET | `/api/hr/salaries` | 薪资列表 |
| POST | `/api/hr/salaries` | 生成工资 |
| GET | `/api/hr/salaries/:id` | 明细 |

## 5. 前后端对应

**后端** (`backend/src/hr/`):

- `repos.rs`, `services.rs`, `handlers.rs` — employees / departments / attendance / salaries

**前端**:

- `features/hr/pages/EmployeeListPage.tsx`
- `features/hr/pages/AttendancePage.tsx`
- `features/hr/pages/SalaryPage.tsx`

## 6. 与其他模块的联动

- 当员工新建时 → `hr.employee.created` → auth 模块创建系统用户
- 当薪资支付时 → `hr.salary.paid` → finance 创建 journal entry
