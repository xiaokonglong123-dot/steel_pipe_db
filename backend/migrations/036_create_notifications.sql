-- 036_create_notifications.sql
-- Notification platform: templates, in-app notifications, user preferences.
--
-- SQLite port: BIGSERIAL -> INTEGER PRIMARY KEY AUTOINCREMENT,
-- TIMESTAMPTZ -> TEXT, NOW() -> datetime('now'), BOOLEAN -> INTEGER (1/0).

CREATE TABLE IF NOT EXISTS notification_templates (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1,
    code        TEXT NOT NULL,
    title       TEXT NOT NULL,
    content_template TEXT NOT NULL,
    channel     TEXT NOT NULL DEFAULT 'in_app',  -- in_app | email
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (tenant_id, code)
);

CREATE TABLE IF NOT EXISTS notifications (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1,
    user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    content     TEXT,
    notify_type TEXT NOT NULL DEFAULT 'system',  -- workflow | finance | inventory | system
    is_read     INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    read_at     TEXT
);

CREATE INDEX IF NOT EXISTS idx_notif_user ON notifications(user_id, is_read);

CREATE TABLE IF NOT EXISTS notification_preferences (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notify_type TEXT NOT NULL,
    channel     TEXT NOT NULL DEFAULT 'in_app',
    enabled     INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (user_id, notify_type, channel)
);
