# D10 Task 1 Report

## Status

DONE

## Files changed

- `src/security.py`
- `src/modules/auth/routers/auth.py`
- `tests/test_login_lockout_audit.py`
- `.superpowers/sdd/2026-08-01-login-lockout-audit/task-1-report.md`

## RED run

Command:

```text
.venv/bin/python -m pytest tests/test_login_lockout_audit.py -v
```

Initial actual result before the production implementation:

```text
collected 6 items

tests/test_login_lockout_audit.py::test_10th_failure_triggers_lockout_audit FAILED
tests/test_login_lockout_audit.py::test_9_failures_no_audit PASSED
tests/test_login_lockout_audit.py::test_locked_attempt_no_new_audit FAILED
tests/test_login_lockout_audit.py::test_ip_429_no_audit PASSED
tests/test_login_lockout_audit.py::test_chain_verifies_after_lockout PASSED
tests/test_login_lockout_audit.py::test_success_clears_failures_no_audit FAILED

3 failed, 3 passed, 1 warning in 4.68s
```

The two expected audit failures were missing audit rows. The third failure was
the standalone fixture reaching token issuance without Rust initialization.

## GREEN run

The two requested follow-up fixes were applied:

- `locked_for={LoginRateLimiter.LOCKOUT_SECONDS:g}s` now renders `locked_for=300s`.
- `rust_auth.init(os.environ["JWT_SECRET_KEY"])` runs immediately after the
  test JWT secret is configured.

Final focused command:

```text
.venv/bin/python -m pytest tests/test_login_lockout_audit.py -v
```

Actual result:

```text
collected 6 items

tests/test_login_lockout_audit.py::test_10th_failure_triggers_lockout_audit PASSED
tests/test_login_lockout_audit.py::test_9_failures_no_audit PASSED
tests/test_login_lockout_audit.py::test_locked_attempt_no_new_audit PASSED
tests/test_login_lockout_audit.py::test_ip_429_no_audit PASSED
tests/test_login_lockout_audit.py::test_chain_verifies_after_lockout PASSED
tests/test_login_lockout_audit.py::test_success_clears_failures_no_audit PASSED

6 passed, 1 warning in 4.15s
```

## Full regression

Command:

```text
.venv/bin/python -m pytest -m "not slow"
```

Actual result:

```text
collected 118 items / 3 deselected / 115 selected

tests/test_admin_audit.py .............
tests/test_audit_route.py ......
tests/test_concurrent_stockout.py ...
tests/test_import_http.py .......
tests/test_login_lockout_audit.py ......
tests/test_panic_recovery.py .......
tests/test_routes_protection.py .................................
tests/test_security.py ..............
tests/test_stock_in_compliance.py ......
tests/test_tubes_routes.py .............
tests/test_upload_limit.py ..
tests/test_used_count.py .....

115 passed, 3 deselected, 2 warnings in 16.86s
```

`test_security.py` and `test_routes_protection.py` passed with no new
failures.

Observed warnings, not failures:

- `PytestConfigWarning`: unknown `asyncio_mode` option.
- `StarletteDeprecationWarning`: `httpx` usage in `starlette.testclient` is
  deprecated; install `httpx2`.

## Concerns

No blocking concerns. The two brief inconsistencies identified during the
initial run were resolved by the explicitly requested follow-up fixes.
