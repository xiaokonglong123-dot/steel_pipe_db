-- 036_create_notifications.sql
-- Notification platform: templates, in-app notifications, user preferences.

CREATE TABLE IF NOT EXISTS notification_templates (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    code        VARCHAR(50) NOT NULL,
    title       VARCHAR(200) NOT NULL,
    content_template TEXT NOT NULL,
    channel     VARCHAR(20) NOT NULL DEFAULT 'in_app',  -- in_app | email
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, code)
);

CREATE TABLE IF NOT EXISTS notifications (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    user_id     BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title       VARCHAR(200) NOT NULL,
    content     TEXT,
    notify_type VARCHAR(50) NOT NULL DEFAULT 'system',  -- workflow | finance | inventory | system
    is_read     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    read_at     TIMESTAMPTZ
);

CREATE INDEX idx_notif_user ON notifications(user_id, is_read);

CREATE TABLE IF NOT EXISTS notification_preferences (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notify_type VARCHAR(50) NOT NULL,
    channel     VARCHAR(20) NOT NULL DEFAULT 'in_app',
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, notify_type, channel)
);
