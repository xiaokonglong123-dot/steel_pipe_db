-- Users table for authentication and RBAC
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

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_role ON users(role);

-- Insert default admin user (password: admin123)
INSERT INTO users (username, password_hash, display_name, role)
VALUES ('admin', '$argon2id$v=19$m=19456,t=2,p=1$+YobHflrRI2qxqqUqIIB8A$0ECSCWpGHdX73H5CVw1n3YYAQJABRnRHQ76Mg3f+ebI', 'Administrator', 'admin');
