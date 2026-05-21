-- Operation logs for audit trail
CREATE TABLE IF NOT EXISTS operation_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    username TEXT,
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id INTEGER,
    details TEXT,
    ip_address TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_operation_logs_user ON operation_logs(user_id);
CREATE INDEX idx_operation_logs_entity ON operation_logs(entity_type, entity_id);
CREATE INDEX idx_operation_logs_created_at ON operation_logs(created_at);
CREATE INDEX idx_operation_logs_action ON operation_logs(action);
