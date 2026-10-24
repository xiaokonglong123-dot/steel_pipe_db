# Task 2 Report: Standards Audit Service and Route Slimming

## What was implemented

- Added `src/modules/warehouse/services/standards_service.py` with audited,
  raw-connection `create_standard`, `update_standard`, and `delete_standard`
  operations, plus `StandardConflictError`.
- Updated `src/modules/warehouse/routers/standards.py` so the three write
  handlers delegate to the service as `create_standard_route`,
  `update_standard_route`, and `delete_standard_route`.
- Appended standards audit tests 9–12 to `tests/test_admin_audit.py` without
  modifying the existing tests 1–8.

## TDD evidence

### RED

Command:

```text
.venv/bin/python -m pytest tests/test_admin_audit.py -k "standard" -v
```

Result excerpt:

```text
tests/test_admin_audit.py::test_create_standard_writes_audit FAILED
tests/test_admin_audit.py::test_update_standard_writes_audit FAILED
tests/test_admin_audit.py::test_delete_standard_writes_audit_with_snapshot FAILED
E       assert None is not None
================== 3 failed, 9 deselected, 1 warning in 1.03s ==================
```

This was expected: the three write operations still had no audit writes, so
each new test failed at `assert row is not None`; the duplicate-grade guard
passed before implementation.

### GREEN

Command:

```text
.venv/bin/python -m pytest tests/test_admin_audit.py -v
```

Result excerpt:

```text
======================== 12 passed, 3 warnings in 1.84s ========================
```

Route regression:

```text
.venv/bin/python -m pytest tests/test_routes_protection.py tests/test_tubes_routes.py -q
46 passed, 1 warning in 5.24s
```

## Full-suite result

Command:

```text
.venv/bin/python -m pytest -m "not slow" -q
```

Summary:

```text
108 passed, 3 deselected, 3 warnings in 10.05s
```

## Files changed

- `src/modules/warehouse/services/standards_service.py`
- `src/modules/warehouse/routers/standards.py`
- `tests/test_admin_audit.py`
- `.superpowers/sdd/2026-08-01-audit-gap/task-2-report.md`

## Self-review findings

- Service operations use `begin_immediate`, perform the business operation and
  `_write_audit` in the same transaction, then commit.
- Duplicate grades roll back and raise `StandardConflictError`, preventing an
  audit row on the conflict path.
- Update and delete use the specified service-level `ValueError` handling.
- Delete snapshots the explicit standard columns before hard deletion.
- The read-only `list_standards` and `edit_standard_form` handlers were left
  unchanged.
- Basedpyright diagnostics were unavailable because the configured server is
  not installed and installation had previously been declined.

## Concerns

No functional concerns. Test runs reported the existing `asyncio_mode` pytest
configuration warning, a Starlette/httpx deprecation warning in the full suite,
and aiosqlite event-loop thread warnings in the audit tests; none caused a test
failure.
