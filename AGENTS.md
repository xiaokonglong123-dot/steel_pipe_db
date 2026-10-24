# AGENTS.md — Ayanami_Rei

**Generated:** 2026-08-01
**Version:** v0.8.0
**Design Authority:** `docs/2026-07-13-api5ct-inventory-design.md`
**Stack:** Python FastAPI + Rust PyO3 + SQLite (WAL) + Jinja2 SSR + Bootstrap 5
**Deployment:** Single worker (`uvicorn --workers 1`). SQLite single-writer.

---

## OVERVIEW

API 5CT seamless pipe & screen pipe inventory management. Tracks stock-in/out, transfers, audits (tamper-evident hash chain), API 5CT compliance on surplus pipes, role-based access. Rust PyO3 module handles JWT auth, compliance calculation, and audit chain hashing as **pure functions with zero state**.

---

## STRUCTURE

> **Modular architecture**: Python app is split into `src/modules/` with nested routers/services per domain module. Future modules (sales, procurement, QC) add sibling directories — no structural refactoring needed. Rust stays flat (cross-cutting concerns, not domain modules). Templates grouped by module to prevent filename collisions.

```
Ayanami_Rei/
├── pyproject.toml          # Python deps (FastAPI, SQLAlchemy, Alembic, etc.)
├── migrations/             # Alembic migration scripts (flat — timestamped files)
│   ├── alembic.ini
│   └── versions/
├── src/                    # FastAPI application
│   ├── main.py             # App factory, startup (rust_auth.init), middleware
│   ├── config.py           # pydantic-settings, env vars, secrets
│   ├── database.py         # SQLAlchemy async engine + PRAGMA listener
│   ├── models.py           # SQLAlchemy ORM models (single file; split at ~300 lines)
│   ├── schemas.py          # pydantic v2 request/response schemas (single file)
│   ├── security.py         # CSRF token generation/validation + JWT dependency injection
│   ├── modules/            # Modular domain grouping
│   │   ├── __init__.py     # register_all_routers(app) — iterates known modules
│   │   ├── auth/           # 验证 (authentication)
│   │   │   ├── routers/auth.py   # JWT login, POST /logout (revoke), POST /auth/refresh (rotate)
│   │   │   └── services/         # (auth calls Rust directly, minimal service layer)
│   │   ├── warehouse/      # 仓库 (inventory management)
│   │   │   ├── routers/
│   │   │   │   ├── tubes.py     # CRUD + stock-out/transfer + TOCTOU guard
│   │   │   │   ├── audit.py     # Audit log query, verify_chain (batched)
│   │   │   │   └── standards.py # API 5CT compliance endpoints
│   │   │   └── services/
│   │   │       ├── tube_service.py    # Business logic for tube operations
│   │   │       ├── import_service.py  # CSV import (pandas, 20MB limit)
│   │   │       ├── standards_service.py  # API 5CT 标准规格 CRUD + 审计（v0.5.0）
│   │   │       ├── check_service.py   # 盘点单持久化、差异应用、recover
│   │   │       ├── location_service.py # 库位 create/rename/deactivate + 审计（2026-10-24）
│   │   │       └── audit_service.py   # Write-audit + Rust compute_hash calls
│   │   └── users/          # 用户管理 (cross-cutting — user CRUD)
│   │       ├── routers/users.py
│   │       └── services/user_service.py
│   ├── templates/          # Jinja2 SSR templates — grouped by module
│   │   ├── base.html       # Shared layout (Bootstrap 5 shell)
│   │   ├── auth/login.html
│   │   ├── dashboard/index.html
│   │   ├── warehouse/      # tube_list, stock_in, stock_out, transfer, check, etc.
│   │   │   ├── check_list.html
│   │   │   ├── check_detail.html
│   │   │   └── check_report.html
│   │   ├── users/          # user_list, user_form
│   │   └── audit/          # audit_logs
│   │                           # (no static/ — Bootstrap 5.3.3 + Icons via jsdelivr CDN)
├── rust-core/              # PyO3 native module (maturin) — flat structure
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Module registration
│       ├── auth.rs         # Argon2 password hash, JWT HS256 sign/verify
│       ├── compliance.rs   # API 5CT compliance validation (pure fn)
│       └── audit.rs        # compute_hash (pure fn), verify_chain_chunk (batched)
└── docs/                   # Design documents
```

**Future modules** (planned — add as sibling directories under `src/modules/`):
- `sales/` — 销售 (orders, pricing, customer management)
- `procurement/` — 采购 (POs, supplier management)
- `qc/` — 质检 (full quality inspection beyond current quality_mark)

---

## WHERE TO LOOK

| Concern | File |
|---|---|
| DB engine, PRAGMAs, connection setup | `src/database.py` |
| ORM models (tubes, users, audit, etc.) | `src/models.py` |
| Business logic: stock-out, transfer, surplus | `src/modules/warehouse/services/tube_service.py` |
| Audit hash chain writes + reads | `src/modules/warehouse/services/audit_service.py` |
| Location create/rename/deactivate (audited) | `src/modules/warehouse/services/location_service.py` |
| CSV import with compliance check | `src/modules/warehouse/services/import_service.py` |
| Rust: JWT, compliance, audit hashing | `rust-core/src/` |
| CSRF token generation | `src/security.py` |
| App factory, middleware, startup | `src/main.py` |
| Alembic migrations | `migrations/versions/` |
| Design authority | `docs/2026-07-13-api5ct-inventory-design.md` |

---

## CONVENTIONS

1. **MUST** deploy as single worker (`--workers 1`). SQLite single-writer — data corruption if >1.
2. **MUST** use `BEGIN IMMEDIATE` for ALL state-changing txns — prevents deadlock with WAL readers.
3. **MUST** use TOCTOU guard pattern: `WHERE status='in_stock' AND quality_mark != 'scrapped'` + check `rows_affected == 1`.
4. **MUST** call `rust_auth.init(secret_key)` once at startup; fail-fast if `JWT_SECRET_KEY` missing.
5. **MUST** keep JWT in httpOnly cookies: access (15min) + refresh (30d). No Bearer header.
6. **MUST** revoke all sessions via `users.token_version`; rotate single session via `users.last_refresh_jti`. Rotation happens at `POST /auth/refresh`; `POST /logout` clears `last_refresh_jti` (revokes refresh), `GET /logout` does NOT exist (405).
7. **MUST** self-implement CSRF: `<input type="hidden" name="csrf_token">` in every Jinja2 form. `require_csrf` also accepts `X-CSRF-Token` header (JSON batch APIs).
8. **MUST** dual-layer login rate limit: IP-level 429 (per `X-Forwarded-For` or client IP) + username-level account lockout.
9. **MUST** enforce FK: `PRAGMA foreign_keys=ON` on EVERY connection via `@event.listens_for`.
10. **MUST** use `ON DELETE RESTRICT` on location FK — prevent orphan tubes.
11. **MUST** soft-delete tubes and locations: `deleted_at` / `is_active`. No physical delete.
12. **MUST** update `locations.used_count` in same txn as business operation; daily reconciliation cron to fix drift.
13. **MUST** write audit log in SAME `BEGIN IMMEDIATE` txn as the operation it records.
14. **MUST** call Rust `compute_hash` (pure fn, no state) for audit chain hash.
15. **MUST** use `sha256("AYANAMI_REI_GENESIS")` as genesis hash.
16. **MUST** batch verify_chain at 20K logs per chunk; use `verify_chain_chunk()`.
17. **MUST** use `with op.batch_alter_table()` in ALL Alembic migrations (SQLite ALTER TABLE is crippled).
18. **MUST** use `sqlite3 .backup` (NOT `cp`) for DB backups; set `busy_timeout=5000`.
19. **MUST** separate seamless vs screen fields via type extension tables (1:1 by `pipe_id`).
20. **MUST** enforce roles: `admin` / `operator` / `viewer` only. Warehouse write routes require `require_operator`; audit verify + standards write + users + location writes require `require_admin`.
21. **MUST** set upload limit to 20MB (override FastAPI default ~2MB).
22. `pipe_id` format: `{SP|SCP}-{grade}-{od}x{wt}-{uuid8}`.
23. Rust modules MUST be pure functions with zero state (no chain tip, no session). Call via `anyio.to_thread.run_sync`.
24. Concurrent stock-out test: `asyncio.gather` + `httpx.AsyncClient(ASGITransport)` → 1 success, N-1 HTTP 409.

---

## ANTI-PATTERNS (THIS PROJECT)

| Anti-pattern | Why |
|---|---|
| `workers > 1` | SQLite single-writer; concurrent writes corrupt data |
| Rust holding chain tip / session state in memory | Drifts across workers; defeats audit integrity |
| Assuming FastAPI has CSRF | It doesn't. Must self-implement. |
| Physical delete of tubes or locations | Breaks audit trail, FK chain |
| Skipping compliance on surplus pipes (盘盈) | All inbound tubes must pass API 5CT check |
| Writing audit outside `BEGIN IMMEDIATE` txn | Audit out-of-sync with business operation |
| Full-table `verify_chain` (no batching) | 20K+ log burst OOMs or blocks connection |
| `COUNT(*)` instead of `used_count` | Full scan on every query; `used_count` is denormalized |
| Hardcoded default admin password | Must fail if `JWT_SECRET_KEY` / admin creds not set |
| Using `token_version` for refresh rotation | Rotates ALL sessions; use `last_refresh_jti` for single-session |

---

## COMMANDS

```bash
# Development server (SINGLE WORKER — mandatory)
uvicorn src.main:app --host 0.0.0.0 --port 8000 --workers 1

# Build Rust native module
maturin develop --release

# Alembic migration
alembic upgrade head

# DB backup (use .backup, NOT cp)
sqlite3 data/inventory.db ".backup data/backup/inventory_$(date +%F).db"

# Lint (must be zero errors; config in pyproject.toml [tool.ruff.lint])
uv run --inexact ruff check

# Concurrent stock-out test
python tests/test_concurrent_stockout.py
```

---

## NOTES

- Genesis audit hash: `sha256("AYANAMI_REI_GENESIS")` — constant lives in `src/models.py` (not Rust); Rust `audit.rs` is a pure function taking `prev_hash`.
- Manual `pipe_id` is weakly validated in `tube_service._validate_pipe_id_format` (SP-/SCP- prefix + 4 parts); failures re-render stock-in form with 422 (was 500, fixed 2026-08-01, D8).
- `busy_timeout=5000` on DB backup command — prevents SQLITE_BUSY under write load.
- Lint is wired (2026-10-24): `uv run --inexact ruff check` must report **zero**. Selection is explicit in `pyproject.toml` (`E4/E7/E9/F/I/UP/B/C4/SIM/RET/RUF/ASYNC/BLE`) — deliberately not `ALL`, because `D`/`ANN` would demand docstring/annotation churn with no runtime benefit. Waived: `RUF001-003` (Chinese comments use fullwidth punctuation), `tests/*` E402 (test files must `sys.path.insert` + set env before importing `src`), and FastAPI's `Depends`/`Form`/`Query`/`File` defaults via `flake8-bugbear.extend-immutable-calls`. `.superpowers/` snapshots are excluded — they are frozen pre-change copies, not source. One deliberate in-code waiver: `# noqa: BLE001` on the batch **stock-in** per-row error collector.
- Test baseline (2026-10-24): 165 passed + 3 deselected via `pytest -m "not slow"`; the 3 slow tests (20K audit stress) pass under `pytest -m slow`. Run with `uv run --inexact pytest -m "not slow" -q` (`--inexact` keeps the maturin-installed `rust_core` extraneous package; a plain `uv sync` removes it).
- 库位写操作（`POST /locations`、`/location/{code}/edit|deactivate`）走 `location_service`：单 `BEGIN IMMEDIATE` + 同事务审计行，失败路径回滚零审计（重复库位 409 / 不存在 404 / 已停用 409 / 重命名名称未变静默 303）；重复与已停用返回 409 并渲染 `location_list.html` 的 `error` 横幅（非 JSON detail）。
- Batch stock-out (`POST /tubes/stock-out/batch`) is **atomic**: one `BEGIN IMMEDIATE`, per-tube TOCTOU guard, any failure rolls the whole batch back (409/403 via app-level handlers), one `Transaction` header + per-tube `TransactionItem` + per-tube audit row. Empty selection = no-op 303. Single-tube route unchanged; batch stock-**in** and CSV import remain per-row partial-success by design.
- Audit covers warehouse tube ops + user management (create_user/update_user/revoke_sessions/toggle_user) + standards writes (create/update/delete with pre-delete snapshot) + location writes (create_location/update_location/deactivate_location, `location_service.py`) + login lockout trigger（第 10 次失败）写审计（action=login_lockout、user_id=NULL、detail=username+ip+failures+locked_for）；锁定期间/IP 层 429 不审计；AuditLog.user_id is the actor (v0.6.0, D10).
- 已知缺口：库位 `deactivate` 单向、无重新启用入口。`/locations*`、`/tubes/{pipe_id}`（详情）、`/tubes/quality`、全部盘点端点均落在 `warehouse/routers/tubes.py`，未拆成独立 router 文件（库位业务逻辑已抽到 `location_service.py`）。
- 盘点语义：`check_sessions`/`check_items` 持久化；盘亏使用 `status='missing'`（新状态值，非 `out_stock`）；确认单在单个 `BEGIN IMMEDIATE` 内原子应用（失败整体回滚、session 保持 open、零审计）；盘盈走 compliance；recover 找回；仅 confirm/recover 写审计。
- Refer to design doc §4.1 for `pipe_id` auto-generation details and overrides.
- Refer to design doc §12 for full TOCTOU guard pseudocode.
- Refer to design doc §3.1 for schema DDL.
