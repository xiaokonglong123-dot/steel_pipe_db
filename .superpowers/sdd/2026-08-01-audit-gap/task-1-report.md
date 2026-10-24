# Task 1 Report: user_service audit integration

## What I implemented

- Created `tests/test_admin_audit.py` with the brief's exact test skeleton and tests 1-8.
- Updated `src/modules/users/services/user_service.py` to:
  - import `_write_audit` alongside `begin_immediate`;
  - require keyword-only `actor_id: int` on `create_user`, `update_user`, `revoke_all_sessions`, and `toggle_active`;
  - write the exact audit entries specified by the brief before each transaction commits.
- Updated `src/modules/users/routers/users.py` to pass `actor_id=user.id` at all four write call sites.

## TDD evidence

### RED

Command:

```text
.venv/bin/python -m pytest tests/test_admin_audit.py -v
```

Observed output excerpt:

```text
collected 8 items
...
==================== 6 failed, 2 passed, 1 warning in 3.11s ====================
```

The five audit-row tests failed on `assert row is not None`, confirming that the routes performed their operations without audit writes. `test_update_user_no_changes_no_audit` failed with `TypeError: update_user() got an unexpected keyword argument 'actor_id'`, confirming that the service signature had not yet been changed. The two zero-audit guard tests passed as expected.

### GREEN

Command:

```text
.venv/bin/python -m pytest tests/test_admin_audit.py -v
```

Observed output excerpt:

```text
======================== 8 passed, 2 warnings in 1.81s =========================
```

## Full-suite result

Command:

```text
.venv/bin/python -m pytest -m "not slow" -q
```

Observed summary:

```text
104 passed, 3 deselected, 4 warnings in 9.96s
```

## Files changed

- `tests/test_admin_audit.py`
- `src/modules/users/services/user_service.py`
- `src/modules/users/routers/users.py`
- `.superpowers/sdd/2026-08-01-audit-gap/task-1-report.md`

## Self-review findings

- The audit writes occur in the existing `BEGIN IMMEDIATE` transaction and before the corresponding commit.
- `AuditLog.user_id` receives the actor ID, while target user IDs remain in the detail strings where specified.
- Password audit detail records only `password_changed=true`; it does not include the generated Argon2 hash.
- The four service parameters were added as keyword-only `actor_id` parameters, and all four router call sites pass the authenticated actor's `user.id`.
- No restructuring or unrelated behavior changes were made.
- The required post-write no-excuse check reported existing `object` annotations and a `ValueError` in `user_service.py`, plus the brief-mandated `asyncio` import in the exact test transcription. These were not changed because the task requires exact transcription and scope-limited changes.

## Concerns

- basedpyright diagnostics were unavailable because the server is not installed and installation was previously declined.
- Pytest emitted the existing `asyncio_mode` configuration warning. The focused GREEN run also emitted an aiosqlite event-loop thread warning; the full regression additionally emitted the existing FastAPI/httpx deprecation warning and two aiosqlite thread warnings. All required tests passed.
