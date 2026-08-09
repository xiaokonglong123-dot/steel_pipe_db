-- 027_create_hr.sql
-- HR module: employees, positions, attendance, attendance rules, salaries,
-- labor contracts. Departments reuse auth.departments (already RBAC-scoped);
-- performance reviews (P2) are intentionally deferred.
--
-- SQLite port: BIGSERIAL -> INTEGER PRIMARY KEY AUTOINCREMENT,
-- TIMESTAMPTZ -> TEXT, NOW() -> datetime('now'), DATE -> TEXT,
-- TIME -> TEXT, BOOLEAN -> INTEGER (1/0), NUMERIC(12,2) -> NUMERIC.

CREATE TABLE IF NOT EXISTS hr_positions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id       INTEGER NOT NULL DEFAULT 1.0,
    department_id   INTEGER,                   -- nullable: generic position
    title           TEXT NOT NULL,
    level           TEXT,
    description     TEXT,
    is_active       INTEGER NOT NULL DEFAULT 1.0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at      TEXT
);

CREATE INDEX IF NOT EXISTS idx_positions_dept ON hr_positions(department_id);

CREATE TABLE IF NOT EXISTS hr_employees (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id       INTEGER NOT NULL DEFAULT 1.0,
    employee_no     TEXT UNIQUE NOT NULL,
    user_id         INTEGER,                   -- optional login account link
    name            TEXT NOT NULL,
    gender          TEXT,
    birth_date      TEXT,                      -- DATE -> TEXT
    id_card         TEXT,
    phone           TEXT,
    email           TEXT,
    department_id   INTEGER,
    position_id     INTEGER,
    hire_date       TEXT NOT NULL,             -- DATE -> TEXT
    probation_end   TEXT,
    status          TEXT NOT NULL DEFAULT 'active',  -- active | on_leave | terminated
    base_salary     REAL DEFAULT 0.0,
    notes           TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at      TEXT
);

CREATE INDEX IF NOT EXISTS idx_employees_dept ON hr_employees(department_id);
CREATE INDEX IF NOT EXISTS idx_employees_status ON hr_employees(status);

CREATE TABLE IF NOT EXISTS hr_attendance_rules (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id         INTEGER NOT NULL DEFAULT 1.0,
    name              TEXT NOT NULL,
    department_id     INTEGER,                 -- nullable: org-wide default
    work_start_time   TEXT NOT NULL DEFAULT '09:00',  -- TIME -> TEXT
    work_end_time     TEXT NOT NULL DEFAULT '18:00',
    grace_minutes     INTEGER NOT NULL DEFAULT 15,    -- late tolerance
    is_active         INTEGER NOT NULL DEFAULT 1.0,
    created_at        TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS hr_attendances (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id     INTEGER NOT NULL REFERENCES hr_employees(id),
    work_date       TEXT NOT NULL,             -- DATE -> TEXT
    check_in        TEXT,                      -- TIMESTAMPTZ -> TEXT
    check_out       TEXT,
    status          TEXT NOT NULL DEFAULT 'normal', -- normal | late | absent | leave
    remark          TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (employee_id, work_date)
);

CREATE INDEX IF NOT EXISTS idx_att_employee ON hr_attendances(employee_id, work_date);

CREATE TABLE IF NOT EXISTS hr_salaries (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id       INTEGER NOT NULL DEFAULT 1.0,
    employee_id     INTEGER NOT NULL REFERENCES hr_employees(id),
    period          TEXT NOT NULL,             -- 'YYYY-MM'
    base_salary     REAL NOT NULL DEFAULT 0.0,
    allowance       REAL NOT NULL DEFAULT 0.0,
    commission      REAL NOT NULL DEFAULT 0.0,
    deduction       REAL NOT NULL DEFAULT 0.0,
    social_security REAL NOT NULL DEFAULT 0.0,
    gross           REAL NOT NULL DEFAULT 0.0,
    net             REAL NOT NULL DEFAULT 0.0,
    status          TEXT NOT NULL DEFAULT 'draft',  -- draft | paid
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (employee_id, period)
);

CREATE INDEX IF NOT EXISTS idx_salaries_period ON hr_salaries(period);

CREATE TABLE IF NOT EXISTS hr_contracts (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id       INTEGER NOT NULL DEFAULT 1.0,
    employee_id     INTEGER NOT NULL REFERENCES hr_employees(id),
    contract_no     TEXT UNIQUE NOT NULL,
    contract_type   TEXT NOT NULL DEFAULT 'fixed',  -- fixed | indefinite | internship
    start_date      TEXT NOT NULL,             -- DATE -> TEXT
    end_date        TEXT,
    status          TEXT NOT NULL DEFAULT 'active',  -- active | expired | terminated
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_contracts_employee ON hr_contracts(employee_id);
