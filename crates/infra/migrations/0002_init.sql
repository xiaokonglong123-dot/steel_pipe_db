-- 0002_init —— 业务基线：11 张表（设计 §4）+ 登录限流
--
-- 时间戳一律由应用写入 RFC3339（微秒精度，+00:00），不用 DB 默认值：
-- 审计哈希把 timestamp 字符串算进链，格式必须唯一确定。
-- 布尔一律 INTEGER 0/1 + CHECK。

-- ---------- 用户 ----------
CREATE TABLE users (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    username         TEXT    NOT NULL UNIQUE,
    password_hash    TEXT    NOT NULL,
    display_name     TEXT    NOT NULL,
    role             TEXT    NOT NULL CHECK (role IN ('admin', 'operator', 'viewer')),
    token_version    INTEGER NOT NULL DEFAULT 0 CHECK (token_version >= 0),
    last_refresh_jti TEXT,
    active           INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    created_at       TEXT    NOT NULL
);

-- ---------- 库位 ----------
CREATE TABLE locations (
    code       TEXT    PRIMARY KEY,
    name       TEXT,
    used_count INTEGER NOT NULL DEFAULT 0 CHECK (used_count >= 0),
    capacity   INTEGER CHECK (capacity IS NULL OR capacity >= 0),
    is_active  INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
);

-- ---------- 管子 ----------
-- location 不设 FK：旧的 CSV 导入允许先写管子再建库位（设计 §4.1 同样无 FK）。
CREATE TABLE tubes (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    pipe_id           TEXT    NOT NULL UNIQUE,
    type              TEXT    NOT NULL CHECK (type IN ('seamless_pipe', 'screen_pipe')),
    outer_diameter_mm REAL    NOT NULL CHECK (outer_diameter_mm > 0),
    wall_thickness_mm REAL    NOT NULL CHECK (wall_thickness_mm > 0),
    length_m          REAL    NOT NULL CHECK (length_m > 0),
    steel_grade       TEXT    NOT NULL,
    thread_type       TEXT,
    api_5ct_cert      TEXT,
    manufacturer      TEXT    NOT NULL,
    batch_no          TEXT,
    heat_no           TEXT,
    heat_treat_method TEXT,
    location          TEXT,
    status            TEXT    NOT NULL CHECK (status IN ('in_stock', 'in_transfer', 'out_stock', 'missing')),
    quality_mark      TEXT    NOT NULL DEFAULT 'normal'
                              CHECK (quality_mark IN ('normal', 'problematic', 'scrapped')),
    quality_note      TEXT,
    stock_in_time     TEXT,
    stock_out_time    TEXT,
    compliance_pass   INTEGER CHECK (compliance_pass IS NULL OR compliance_pass IN (0, 1)),
    deleted_at        TEXT,
    created_at        TEXT    NOT NULL,
    updated_at        TEXT    NOT NULL
);

CREATE INDEX ix_tubes_status       ON tubes (status);
CREATE INDEX ix_tubes_location     ON tubes (location);
CREATE INDEX ix_tubes_quality_mark ON tubes (quality_mark);
CREATE INDEX ix_tubes_steel_grade  ON tubes (steel_grade);
CREATE INDEX ix_tubes_batch_no     ON tubes (batch_no);
CREATE INDEX ix_tubes_heat_no      ON tubes (heat_no);
CREATE INDEX ix_tubes_deleted_at   ON tubes (deleted_at);

-- ---------- 类型扩展表（1:1） ----------
CREATE TABLE seamless_pipe_specs (
    pipe_id           TEXT PRIMARY KEY
                           REFERENCES tubes (pipe_id) ON UPDATE CASCADE ON DELETE CASCADE,
    surface_treatment TEXT,
    end_protector     TEXT
);

CREATE TABLE screen_pipe_specs (
    pipe_id             TEXT PRIMARY KEY
                             REFERENCES tubes (pipe_id) ON UPDATE CASCADE ON DELETE CASCADE,
    screen_type         TEXT,
    slot_width_mm       REAL CHECK (slot_width_mm IS NULL OR slot_width_mm > 0),
    slot_density        REAL CHECK (slot_density IS NULL OR slot_density >= 0),
    filtration_rating_um REAL CHECK (filtration_rating_um IS NULL OR filtration_rating_um > 0),
    base_pipe_grade     TEXT,
    slot_pattern        TEXT
);

-- ---------- 单据 ----------
CREATE TABLE transactions (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    type         TEXT    NOT NULL CHECK (type IN ('in', 'out', 'move', 'check')),
    operator_id  INTEGER NOT NULL,
    reference_no TEXT,
    remark       TEXT,
    created_at   TEXT    NOT NULL
);
CREATE INDEX ix_transactions_created_at ON transactions (created_at);
CREATE INDEX ix_transactions_type       ON transactions (type);

-- pipe_id 刻意无 FK（设计 §4.5）：防级联删管，历史单据必须留档。
CREATE TABLE transaction_items (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_id INTEGER NOT NULL REFERENCES transactions (id) ON DELETE CASCADE,
    pipe_id        TEXT    NOT NULL,
    source_location TEXT,
    target_location TEXT
);
CREATE INDEX ix_transaction_items_transaction_id ON transaction_items (transaction_id);
CREATE INDEX ix_transaction_items_pipe_id        ON transaction_items (pipe_id);

-- ---------- 审计链 ----------
CREATE TABLE audit_logs (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id      INTEGER,
    action       TEXT    NOT NULL,
    detail       TEXT,
    prev_hash    TEXT    NOT NULL CHECK (length(prev_hash) = 64),
    current_hash TEXT    NOT NULL UNIQUE CHECK (length(current_hash) = 64),
    ts           TEXT    NOT NULL,
    created_at   TEXT    NOT NULL
);
CREATE INDEX ix_audit_logs_action ON audit_logs (action);
CREATE INDEX ix_audit_logs_ts     ON audit_logs (ts);

-- ---------- 标准规格 ----------
CREATE TABLE standard_specs (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
    grade                 TEXT    NOT NULL,
    outer_diameter_mm     REAL    CHECK (outer_diameter_mm IS NULL OR outer_diameter_mm > 0),
    min_wall_thickness_mm REAL    CHECK (min_wall_thickness_mm IS NULL OR min_wall_thickness_mm > 0),
    max_wall_thickness_mm REAL    CHECK (max_wall_thickness_mm IS NULL OR max_wall_thickness_mm > 0),
    length_tolerance_mm   REAL    CHECK (length_tolerance_mm IS NULL OR length_tolerance_mm >= 0),
    source                TEXT,
    CONSTRAINT uq_standard_specs_grade UNIQUE (grade),
    CONSTRAINT ck_standard_specs_wall_range
        CHECK (min_wall_thickness_mm IS NULL
            OR max_wall_thickness_mm IS NULL
            OR min_wall_thickness_mm <= max_wall_thickness_mm)
);

-- ---------- 盘点 ----------
CREATE TABLE check_sessions (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    location     TEXT    NOT NULL,
    status       TEXT    NOT NULL DEFAULT 'open'
                         CHECK (status IN ('open', 'confirmed', 'cancelled')),
    created_by   INTEGER NOT NULL,
    confirmed_by INTEGER,
    created_at   TEXT    NOT NULL,
    confirmed_at TEXT,
    remark       TEXT
);
CREATE INDEX ix_check_sessions_location ON check_sessions (location);
CREATE INDEX ix_check_sessions_status   ON check_sessions (status);

CREATE TABLE check_items (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    check_id        INTEGER NOT NULL REFERENCES check_sessions (id) ON DELETE CASCADE,
    pipe_id         TEXT    NOT NULL,
    result          TEXT    NOT NULL CHECK (result IN ('ok', 'missing', 'moved', 'surplus')),
    actual_location TEXT,
    surplus_data    TEXT,
    created_at      TEXT    NOT NULL
);
CREATE INDEX ix_check_items_check_id ON check_items (check_id);
CREATE INDEX ix_check_items_pipe_id  ON check_items (pipe_id);

-- ---------- 登录限流（A23：落库，重启不清零） ----------
CREATE TABLE login_attempts (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    scope        TEXT    NOT NULL CHECK (scope IN ('ip', 'username')),
    subject      TEXT    NOT NULL,
    attempted_at TEXT    NOT NULL,
    succeeded    INTEGER NOT NULL DEFAULT 0 CHECK (succeeded IN (0, 1))
);
CREATE INDEX ix_login_attempts_scope_subject ON login_attempts (scope, subject, attempted_at);
