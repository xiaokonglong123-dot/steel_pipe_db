-- 0001_init —— 迁移基线：审计链尾
--
-- 方言：SQLite。业务表（11 张 / 88 列）在 0002 落地，见
-- docs/2026-11-17-rewrite-reference.md §4。
--
-- 链尾表强制单行（id = 1）；写入协议为 BEGIN IMMEDIATE + 读链尾，
-- 见 crates/infra 的 WRITE_TXN_BEGIN（M3 落地）。

CREATE TABLE audit_chain_tip (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    last_seq   INTEGER NOT NULL CHECK (last_seq >= 0),
    last_hash  TEXT    NOT NULL CHECK (
                   length(last_hash) = 64
                   AND last_hash GLOB '[0-9a-f]*'
               ),
    updated_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- 创世：sha256("AYANAMI_REI_GENESIS")，与旧实现一致。
INSERT INTO audit_chain_tip (id, last_seq, last_hash)
VALUES (1, 0, '2e3874e8f41f106cc35d41312497bf350acebadc6ab2ebf6728631a9c180cd40');
