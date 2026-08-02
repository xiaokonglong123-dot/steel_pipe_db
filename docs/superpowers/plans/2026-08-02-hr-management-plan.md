# HR Management Implementation Plan

**Goal:** Build HR module with employees, departments, attendance, salary, contracts management.

**Architecture:** New `crates/hr/` in Rust. PostgreSQL schema `hr`. Frontend `features/hr/` with employee list/form, attendance table, salary polygon page.

---

### Task 1: Create `hr` schema tables

```sql
CREATE TABLE hr.employees (id BIGSERIAL PRIMARY KEY, user_id BIGINT, employee_no VARCHAR(50) UNIQUE, hire_date DATE, status VARCHAR(20), ...);
CREATE TABLE hr.attendances (id BIGSERIAL, clock_in TIMESTAMPTZ, ...);
CREATE TABLE hr.salaries (id, month DATE, gross NUMERIC, net NUMERIC, status VARCHAR);
```

### Task 2: Employee CRUD service + API

Pattern: POST `/api/hr/employees`, etc.

### Task 3: Attendance service

### Task 4: Salary service + export to finance journal

### Task 5: Frontend (use same patterns)

Employee page → list with DataTable, form page, detail page. Attendance page, salary page.