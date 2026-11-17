# F — 审计链规格

来源：`tests/test_audit_chain_20k.py`、`test_audit_route.py`、`test_panic_recovery.py`、
`test_admin_audit.py`，旧 `rust-core/src/audit.rs`、`audit_service.py`，设计 §12.3。

| # | 规格 | 状态 |
| --- | --- | --- |
| F1 | `current_hash = sha256(prev_hash ‖ action ‖ detail ‖ timestamp)`，十六进制小写 | ✅ M1（黄金向量锁定旧实现） |
| F2 | 创世哈希 = `sha256("AYANAMI_REI_GENESIS")`，链尾表与常量双处一致 | ✅ M0 |
| F3 | 链尾单行（`CHECK (id = 1)`），事务内推进，不驻留内存 | ✅ M1 |
| F4 | 业务写与审计行同一 `BEGIN IMMEDIATE` 事务；回滚时链尾不动 | ✅ M1 |
| F5 | 分批校验：单批 ≤ 20 000 条，跨批用上一批 tip 续接 | ✅ M1（`verify_chunk`） |
| F6 | 篡改 `detail` → 定位到具体索引并报 `HashMismatch` | ✅ M1 |
| F7 | 断链（缺条目 / prev 不衔接）→ `Discontinuous` | ✅ M1 |
| F8 | 每种写操作写**恰好一条**审计：入库、出库、批量出库（每根一条）、调拨、质量标注、软删、库位、标准、用户 | ✅ M1–M3 |
| F9 | `detail` 必须由 `audit::detail()` 生成（紧凑 JSON，字段顺序固定） | ✅ M1 |
| F10 | `steelpipe audit verify [--batch N]` 分批校验全链，尾与链尾表比对 | ✅ M5 |
| F11 | `steelpipe reconcile` 检测并修复 `used_count` 漂移（修复写审计） | ✅ M5 |

## 与旧实现的差异

1. 旧实现链尾从 `audit_logs ORDER BY id DESC LIMIT 1` 推导；新实现用单行 `audit_chain_tip`，少一次全表扫描，也避免空表歧义。
2. 旧实现 `verify_token` 类遗留 API 不再保留。
