## Summary

Implemented the D11 inventory-check skeleton: raw-connection check-session service operations, eight inventory-check routes, SSR list/detail/report templates, and the 14 focused tests for Task 2. The controller now applies CSRF cookies/tokens to every check render path, enforces operator authorization on all inventory writes, and safely falls back to the check list for missing sessions; confirm/recover business logic remains unimplemented.

## Files changed

- `/home/yzp/Documents/Ayanami_Rei/src/modules/warehouse/services/check_service.py`
- `/home/yzp/Documents/Ayanami_Rei/src/modules/warehouse/routers/tubes.py`
- `/home/yzp/Documents/Ayanami_Rei/src/templates/warehouse/check_list.html`
- `/home/yzp/Documents/Ayanami_Rei/src/templates/warehouse/check_detail.html`
- `/home/yzp/Documents/Ayanami_Rei/src/templates/warehouse/check_report.html`
- `/home/yzp/Documents/Ayanami_Rei/tests/test_check.py`
- `/home/yzp/Documents/Ayanami_Rei/.superpowers/sdd/2026-08-01-check/task-2-report.md`

## TDD evidence

RED command:

```text
$ .venv/bin/python -m pytest tests/test_check.py -v
============================= test session starts ==============================
platform linux -- Python 3.14.6, pytest-9.1.1, pluggy-1.6.0 -- /home/yzp/Documents/Ayanami_Rei/.venv/bin/python
plugins: anyio-4.14.2
collecting ... collected 14 items

tests/test_check.py::test_1_viewer_get_check_list_forbidden FAILED       [  7%]
tests/test_check.py::test_2_viewer_post_check_create_forbidden FAILED    [ 14%]
tests/test_check.py::test_3_operator_create_check FAILED                 [ 21%]
tests/test_check.py::test_4_mark_ok FAILED                               [ 28%]
tests/test_check.py::test_5_mark_missing FAILED                          [ 35%]
tests/test_check.py::test_6_mark_moved_without_location_422 FAILED       [ 42%]
tests/test_check.py::test_7_mark_moved_same_location_409 FAILED           [ 50%]
tests/test_check.py::test_8_mark_moved_valid FAILED                      [ 57%]
tests/test_check.py::test_9_mark_upsert_updates_single_row FAILED         [ 64%]
tests/test_check.py::test_10_add_surplus_creates_item_with_preview_pipe_id FAILED [ 71%]
tests/test_check.py::test_11_delete_surplus_item FAILED                  [ 78%]
tests/test_check.py::test_18a_cancel_open_session FAILED                 [ 85%]
tests/test_check.py::test_12_mark_non_open_session_409 FAILED            [ 92%]
tests/test_check.py::test_13_operator_confirm_forbidden FAILED           [100%]

=============================== warnings summary ===============================
.venv/lib/python3.14/site-packages/_pytest/config/__init__.py:1464
  /home/yzp/Documents/Ayanami_Rei/.venv/lib/python3.14/site-packages/_pytest/config/__init__.py:1464: PytestConfigWarning: Unknown config option: asyncio_mode

======================== 14 failed, 1 warning in 5.10s =========================
```

GREEN command:

```text
$ .venv/bin/python -m pytest tests/test_check.py -v
============================= test session starts ==============================
platform linux -- Python 3.14.6, pytest-9.1.1, pluggy-1.6.0 -- /home/yzp/Documents/Ayanami_Rei/.venv/bin/python
plugins: anyio-4.14.2
collecting ... collected 14 items

tests/test_check.py::test_1_viewer_get_check_list_forbidden PASSED       [  7%]
tests/test_check.py::test_2_viewer_post_check_create_forbidden PASSED    [ 14%]
tests/test_check.py::test_3_operator_create_check PASSED                 [ 21%]
tests/test_check.py::test_4_mark_ok PASSED                               [ 28%]
tests/test_check.py::test_5_mark_missing PASSED                          [ 35%]
tests/test_check.py::test_6_mark_moved_without_location_422 PASSED       [ 42%]
tests/test_check.py::test_7_mark_moved_same_location_409 PASSED           [ 50%]
tests/test_check.py::test_8_mark_moved_valid PASSED                      [ 57%]
tests/test_check.py::test_9_mark_upsert_updates_single_row PASSED         [ 64%]
tests/test_check.py::test_10_add_surplus_creates_item_with_preview_pipe_id PASSED [ 71%]
tests/test_check.py::test_11_delete_surplus_item PASSED                  [ 78%]
tests/test_check.py::test_18a_cancel_open_session PASSED                 [ 85%]
tests/test_check.py::test_12_mark_non_open_session_409 PASSED             [ 92%]
tests/test_check.py::test_13_operator_confirm_forbidden PASSED           [100%]

======================== 14 passed, 1 warning in 4.43s =========================
```

## Regression evidence

Full regression:

```text
$ .venv/bin/python -m pytest -m "not slow"
collected 132 items / 3 deselected / 129 selected
tests/test_admin_audit.py .............                                  [ 10%]
tests/test_audit_route.py ......                                         [ 14%]
tests/test_check.py ..............                                       [ 25%]
tests/test_concurrent_stockout.py ...                                    [ 27%]
tests/test_import_http.py .......                                        [ 33%]
tests/test_login_lockout_audit.py ......                                 [ 37%]
tests/test_panic_recovery.py .......                                     [ 43%]
tests/test_routes_protection.py .................................        [ 68%]
tests/test_security.py ..............                                    [ 79%]
tests/test_stock_in_compliance.py ......                                 [ 84%]
tests/test_tubes_routes.py .............                                 [ 94%]
tests/test_upload_limit.py ..                                            [ 96%]
tests/test_used_count.py .....                                           [100%]

================ 129 passed, 3 deselected, 2 warnings in 17.72s ================
```

Startup smoke:

```text
$ .venv/bin/python -c "from src.main import app; print('ok')"
ok
```

Manual invalid-session verification:

```text
GET /tubes/check/99999 200 csrf_nonempty=True set_cookie=True
GET /tubes/check/99999/report 200 csrf_nonempty=True set_cookie=True
POST /tubes/check/99999/items 409 csrf_nonempty=True set_cookie=True
POST /tubes/check/99999/surplus 409 csrf_nonempty=True set_cookie=True
POST /tubes/check/99999/surplus/1/delete 409 csrf_nonempty=True set_cookie=True
POST /tubes/check/99999/cancel 409 csrf_nonempty=True set_cookie=True
```

## Concerns

The brief’s test 13 requires operator confirmation to return 403 while also saying the confirm route must not exist until Task 3. To satisfy the required 14-test GREEN result without applying confirmation behavior, Task 2 includes an admin-only authorization stub that returns 501 for admins; Task 3 must replace that stub with the real confirm implementation. The supplied test block also omitted several `asyncio.run()` calls, used `Tube(type=...)` although the model attribute is `type_`, deleted the SQLite file while engines were open, and accessed `detail.items` through Jinja’s dict-method collision; the test/template harness contains only those minimal corrections.
