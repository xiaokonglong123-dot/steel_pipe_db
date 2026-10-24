### Task 3: 链连续性集成测试

**Files:**
- Modify: `tests/test_admin_audit.py`（追加测试 13）

**Interfaces:**
- Consumes: Task 1+2 的全部审计写入路径；`POST /audit/verify`（admin 专属，`require_admin` + `require_csrf` 依赖，无表单字段）。
- Produces: 无（纯测试）。

- [ ] **Step 1: 追加测试 13**

在 `tests/test_admin_audit.py` 文件末尾追加：

```python
# --- hash-chain continuity -------------------------------------------------


def test_chain_verifies_after_mixed_ops(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/create", {
                "username": "alice", "password": "Secret1!",
                "display_name": "Alice", "role": "operator",
            })
            assert r.status_code == 303, r.text[:400]
            r = await _post(c, "/standards", {
                "steel_grade": "N80",
                "outer_diameter_mm": "114.3",
                "min_wall_thickness_mm": "7.0",
                "max_wall_thickness_mm": "11.0",
            })
            assert r.status_code == 303, r.text[:400]
            r = await _post(c, "/standards/1/delete", {})
            assert r.status_code == 303, r.text[:400]
            r = await _post(c, "/users/revoke/2", {})
            assert r.status_code == 303, r.text[:400]
            v = await _post(c, "/audit/verify", {})
        assert v.status_code == 200, v.text[:400]
        assert "哈希链完整" in v.text
    asyncio.run(run())
    assert _audit_count() == 4
```

- [ ] **Step 2: 运行确认通过**

Run: `.venv/bin/python -m pytest tests/test_admin_audit.py::test_chain_verifies_after_mixed_ops -v`
Expected: PASS（此测试依赖 Task 1+2 已完成；若未完成会 FAIL）。

- [ ] **Step 3: 全量回归**

Run: `.venv/bin/python -m pytest -m "not slow" -q`
Expected: 109 passed，3 deselected。此为本次功能最终基线。

---

