-- 027_create_hr.sql
-- HR module: employees, positions, attendance, attendance rules, salaries,
-- labor contracts. Departments reuse auth.departments (already RBAC-scoped);
-- performance reviews (P2) are intentionally deferred.

CREATE TABLE IF NOT EXISTS hr_positions (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       BIGINT NOT NULL DEFAULT 1,
    department_id   BIGINT,                        -- nullable: generic position
    title           VARCHAR(100) NOT NULL,
    level           VARCHAR(50),
    description     TEXT,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

CREATE INDEX idx_positions_dept ON hr_positions(department_id);

CREATE TABLE IF NOT EXISTS hr_employees (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       BIGINT NOT NULL DEFAULT 1,
    employee_no     VARCHAR(50) UNIQUE NOT NULL,
    user_id         BIGINT,                        -- optional login account link
    name            VARCHAR(100) NOT NULL,
    gender          VARCHAR(10),
    birth_date      DATE,
    id_card         VARCHAR(30),
    phone           VARCHAR(30),
    email           VARCHAR(100),
    department_id   BIGINT,
    position_id     BIGINT,
    hire_date       DATE NOT NULL,
    probation_end   DATE,
    status          VARCHAR(20) NOT NULL DEFAULT 'active',  -- active | on_leave | terminated
    base_salary     NUMERIC(12,2) DEFAULT 0,
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

CREATE INDEX idx_employees_dept ON hr_employees(department_id);
CREATE INDEX idx_employees_status ON hr_employees(status);

CREATE TABLE IF NOT EXISTS hr_attendance_rules (
    id                BIGSERIAL PRIMARY KEY,
    tenant_id         BIGINT NOT NULL DEFAULT 1,
    name              VARCHAR(100) NOT NULL,
    department_id     BIGINT,                      -- nullable: org-wide default
    work_start_time   TIME NOT NULL DEFAULT '09:00',
    work_end_time     TIME NOT NULL DEFAULT '18:00',
    grace_minutes     INT NOT NULL DEFAULT 15,     -- late tolerance
    is_active         BOOLEAN NOT NULL DEFAULT TRUE,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS hr_attendances (
    id              BIGSERIAL PRIMARY KEY,
    employee_id     BIGINT NOT NULL REFERENCES hr_employees(id),
    work_date       DATE NOT NULL,
    check_in        TIMESTAMPTZ,
    check_out       TIMESTAMPTZ,
    status          VARCHAR(20) NOT NULL DEFAULT 'normal', -- normal | late | absent | leave
    remark          TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (employee_id, work_date)
);

CREATE INDEX idx_att_employee ON hr_attendances(employee_id, work_date);

CREATE TABLE IF NOT EXISTS hr_salaries (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       BIGINT NOT NULL DEFAULT 1,
    employee_id     BIGINT NOT NULL REFERENCES hr_employees(id),
    period          VARCHAR(7) NOT NULL,           -- 'YYYY-MM'
    base_salary     NUMERIC(12,2) NOT NULL DEFAULT 0,
    allowance       NUMERIC(12,2) NOT NULL DEFAULT 0,
    commission      NUMERIC(12,2) NOT NULL DEFAULT 0,
    deduction       NUMERIC(12,2) NOT NULL DEFAULT 0,
    social_security NUMERIC(12,2) NOT NULL DEFAULT 0,
    gross           NUMERIC(12,2) NOT NULL DEFAULT 0,
    net             NUMERIC(12,2) NOT NULL DEFAULT 0,
    status          VARCHAR(20) NOT NULL DEFAULT 'draft',  -- draft | paid
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (employee_id, period)
);

CREATE INDEX idx_salaries_period ON hr_salaries(period);

CREATE TABLE IF NOT EXISTS hr_contracts (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       BIGINT NOT NULL DEFAULT 1,
    employee_id     BIGINT NOT NULL REFERENCES hr_employees(id),
    contract_no     VARCHAR(50) UNIQUE NOT NULL,
    contract_type   VARCHAR(20) NOT NULL DEFAULT 'fixed',  -- fixed | indefinite | internship
    start_date      DATE NOT NULL,
    end_date        DATE,
    status          VARCHAR(20) NOT NULL DEFAULT 'active',  -- active | expired | terminated
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_contracts_employee ON hr_contracts(employee_id);
