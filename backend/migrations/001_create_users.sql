-- Users table for authentication and RBAC
-- 001_create_users.sql
-- System users table with 4 RBAC roles: admin, warehouse, qc, sales.
-- Passwords are Argon2id-hashed (never stored in plain text).
-- Soft delete via deleted_at column.
--
-- SQLite port notes:
--   BIGSERIAL        -> INTEGER PRIMARY KEY AUTOINCREMENT
--   TIMESTAMPTZ      -> TEXT (stored as 'YYYY-MM-DD HH:MM:SS' UTC via datetime('now'))
--   NOW()            -> datetime('now')
--   BOOLEAN          -> INTEGER (1/0)
--
-- PRAGMA foreign_keys is a per-connection setting in SQLite (default OFF).
-- It is enabled here so the REFERENCES clauses in this and later migrations
-- are validated on the migration connection. Runtime connections must set it
-- explicitly (e.g. SqliteConnectOptions::foreign_keys(true)) to enforce FKs
-- after migrations finish.
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    display_name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'warehouse' CHECK (role IN ('admin', 'warehouse', 'qc', 'sales')),
    email TEXT,
    phone TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);

-- NOTE: No seed admin is inserted here. The first admin user is bootstrapped
-- at application startup from ADMIN_USERNAME / ADMIN_PASSWORD env vars
-- (see main.rs bootstrap_admin()). This avoids shipping a hardcoded credential
-- that every deployment inherits.
