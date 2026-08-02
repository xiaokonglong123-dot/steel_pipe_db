-- Users table for authentication and RBAC
-- 001_create_users.sql
-- System users table with 4 RBAC roles: admin, warehouse, qc, sales.
-- Passwords are Argon2id-hashed (never stored in plain text).
-- Soft delete via deleted_at column.
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    display_name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'warehouse' CHECK (role IN ('admin', 'warehouse', 'qc', 'sales')),
    email TEXT,
    phone TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_role ON users(role);

-- NOTE: No seed admin is inserted here. The first admin user is bootstrapped
-- at application startup from ADMIN_USERNAME / ADMIN_PASSWORD env vars
-- (see main.rs bootstrap_admin()). This avoids shipping a hardcoded credential
-- that every deployment inherits.
