# HR Management Implementation Plan

**Goal:** Build HR module with employees, departments, attendance, salary, contracts management.

**Architecture:** New `backend/src/hr/` in Rust. SQLite tables with `hr_` prefix. Frontend `features/hr/` with employee list/form, attendance table, salary page.

**Database:** SQLite3 (`sqlite://data/erp.db?mode=rwc`)

---

### Task 1: Create `hr` tables

```sql
CREATE TABLE IF NOT EXISTS hr_employees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER, employee_no TEXT UNIQUE, hire_date TEXT, status TEXT, ...
);
CREATE TABLE IF NOT EXISTS hr_attendances (
    id INTEGER PRIMARY KEY AUTOINCREMENT, clock_in TEXT, ...
);
CREATE TABLE IF NOT EXISTS hr_salaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    month TEXT, gross REAL, net REAL, status TEXT
);
```

### Task 2: Employee CRUD service + API

Pattern: POST `/api/hr/employees`, etc.

### Task 3: Attendance service

### Task 4: Salary service + export to finance journal

### Task 5: Frontend (use same patterns)

Employee page → list with DataTable, form page, detail page. Attendance page, salary page.
