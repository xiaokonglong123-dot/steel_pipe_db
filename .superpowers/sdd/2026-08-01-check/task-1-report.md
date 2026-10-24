## Summary

Added the D11 inventory-check migration and ORM models, and mechanically extracted stock-in persistence into `_insert_tube_with_compliance` while preserving the existing `stock_in` contract and transaction behavior.

## Files changed

- `/home/yzp/Documents/Ayanami_Rei/migrations/versions/20260801_000000_add_check_tables.py`
- `/home/yzp/Documents/Ayanami_Rei/src/models.py`
- `/home/yzp/Documents/Ayanami_Rei/src/modules/warehouse/services/tube_service.py`

## Test evidence

Migration command:

```text
INFO  [alembic.runtime.migration] Context impl SQLiteImpl.
INFO  [alembic.runtime.migration] Will assume non-transactional DDL.
INFO  [alembic.runtime.migration] Running upgrade  -> 20260713_083945_initial, Initial schema — creates all 9 tables with indexes.
INFO  [alembic.runtime.migration] Running upgrade 20260713_083945_initial -> 20260713_084000_add_audit_ts, Add ts column to audit_logs for deterministic hash verification.
INFO  [alembic.runtime.migration] Running upgrade 20260713_084000_add_audit_ts -> 8ce24f163996, add unique on standard_specs.grade
INFO  [alembic.runtime.migration] Running upgrade 8ce24f163996 -> 20260801_000000_add_check_tables, Add check_sessions + check_items for inventory check (盘点, D11).
```

Database listing and cleanup:

```text
-rw-r--r-- 1 yzp yzp 126976  8月 1日 09:22 data/test_migration_check.db
```

Focused suite:

```text
============================= test session starts ==============================
platform linux -- Python 3.14.6, pytest-9.1.1, pluggy-1.6.0 -- /home/yzp/Documents/Ayanami_Rei/.venv/bin/python
cachedir: .pytest_cache
rootdir: /home/yzp/Documents/Ayanami_Rei
configfile: pyproject.toml
plugins: anyio-4.14.2
collecting ... collected 29 items

tests/test_tubes_routes.py::test_post_stock_in_success_redirects_303 PASSED [  3%]
tests/test_tubes_routes.py::test_post_stock_in_conflict_pipe_id_returns_409 PASSED [  6%]
tests/test_tubes_routes.py::test_post_stock_in_malformed_manual_pipe_id_returns_422 PASSED [ 10%]
tests/test_tubes_routes.py::test_post_stock_out_success_redirects_303 PASSED [ 13%]
tests/test_tubes_routes.py::test_post_stock_out_already_out_returns_409 PASSED [ 17%]
tests/test_tubes_routes.py::test_post_stock_out_missing_csrf_rejected PASSED [ 20%]
tests/test_tubes_routes.py::test_post_batch_stock_in_returns_200 PASSED  [ 24%]
tests/test_tubes_routes.py::test_post_batch_stock_in_missing_csrf_rejected PASSED [ 27%]
tests/test_tubes_routes.py::test_post_transfer_success_redirects_303 PASSED [ 31%]
tests/test_tubes_routes.py::test_post_transfer_same_location_returns_409 PASSED [ 34%]
tests/test_tubes_routes.py::test_post_transfer_not_in_stock_returns_409 PASSED [ 37%]
tests/test_tubes_routes.py::test_post_transfer_missing_csrf_rejected PASSED [ 41%]
tests/test_tubes_routes.py::test_list_tubes_excludes_soft_deleted PASSED [ 44%]
tests/test_stock_in_compliance.py::test_stock_in_compliant_persists_true PASSED [ 48%]
tests/test_stock_in_compliance.py::test_stock_in_wall_thickness_out_of_range_raises PASSED [ 51%]
tests/test_stock_in_compliance.py::test_stock_in_outer_diameter_off_raises PASSED [ 55%]
tests/test_stock_in_compliance.py::test_stock_in_unknown_grade_raises PASSED [ 58%]
tests/test_stock_in_compliance.py::test_stock_in_caller_supplied_compliance_skips_rust PASSED [ 62%]
tests/test_stock_in_compliance.py::test_post_stock_in_noncompliant_returns_422 PASSED [ 65%]
tests/test_import_http.py::test_get_import_form_ok PASSED                [ 68%]
tests/test_import_http.py::test_post_import_compliant_row_ok PASSED      [ 72%]
tests/test_import_http.py::test_post_import_too_large_413 PASSED         [ 75%]
tests/test_import_http.py::test_post_import_missing_columns_400 PASSED   [ 79%]
tests/test_import_http.py::test_post_import_noncompliant_row_collects_error PASSED [ 82%]
tests/test_import_http.py::test_post_import_too_many_rows_400 PASSED     [ 86%]
tests/test_import_http.py::test_import_service_at_row_limit_not_rejected PASSED [ 89%]
tests/test_concurrent_stockout.py::test_concurrent_stock_out PASSED      [ 93%]
tests/test_concurrent_stockout.py::test_used_count_decremented PASSED    [ 96%]
tests/test_concurrent_stockout.py::test_problematic_requires_admin PASSED [100%]

=============================== warnings summary ===============================
.venv/lib/python3.14/site-packages/_pytest/config/__init__.py:1464
  /home/yzp/Documents/Ayanami_Rei/.venv/lib/python3.14/site-packages/_pytest/config/__init__.py:1464: PytestConfigWarning: Unknown config option: asyncio_mode

    self._warn_or_fail_if_strict("Unknown config option: {key}\n")

-- Docs: https://docs.pytest.org/en/stable/how-to/capture-warnings.html
======================== 29 passed, 1 warning in 4.30s =========================
```

Full non-slow suite:

```text
============================= test session starts ==============================
platform linux -- Python 3.14.6, pytest-9.1.1, pluggy-1.6.0
rootdir: /home/yzp/Documents/Ayanami_Rei
configfile: pyproject.toml
testpaths: tests
plugins: anyio-4.14.2
collected 118 items / 3 deselected / 115 selected

tests/test_admin_audit.py .............                                  [ 11%]
tests/test_audit_route.py ......                                         [ 16%]
tests/test_concurrent_stockout.py ...                                    [ 19%]
tests/test_import_http.py .......                                        [ 25%]
tests/test_login_lockout_audit.py ......                                 [ 30%]
tests/test_panic_recovery.py .......                                     [ 36%]
tests/test_routes_protection.py .................................        [ 65%]
tests/test_security.py ..............                                    [ 77%]
tests/test_stock_in_compliance.py ......                                 [ 82%]
tests/test_tubes_routes.py .............                                 [ 93%]
tests/test_upload_limit.py ..                                            [ 95%]
tests/test_used_count.py .....                                           [100%]

=============================== warnings summary ===============================
.venv/lib/python3.14/site-packages/_pytest/config/__init__.py:1464
  /home/yzp/Documents/Ayanami_Rei/.venv/lib/python3.14/site-packages/_pytest/config/__init__.py:1464: PytestConfigWarning: Unknown config option: asyncio_mode

    self._warn_or_fail_if_strict("Unknown config option: {key}\n")

.venv/lib/python3.14/site-packages/fastapi/testclient.py:1
  /home/yzp/Documents/Ayanami_Rei/.venv/lib/python3.14/site-packages/fastapi/testclient.py:1: StarletteDeprecationWarning: Using `httpx` with `starlette.testclient` is deprecated; install `httpx2` instead.
    from starlette.testclient import TestClient as TestClient  # noqa

-- Docs: https://docs.pytest.org/en/stable/how-to/capture-warnings.html
================ 115 passed, 3 deselected, 2 warnings in 15.64s ================
```

## Concerns

None.
