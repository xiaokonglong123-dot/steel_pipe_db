## Task 3: confirm 原子应用 + recover + 测试 14-17/18b/19/20

### Files
```
改 src/modules/warehouse/services/check_service.py      # 追加 confirm_check / recover_missing
改 src/modules/warehouse/routers/tubes.py               # 追加 confirm / recover 2 路由
改 tests/test_check.py                                  # 追加测试 14-17、18b、19、20、20b（8 项）
```

### Interfaces

```python
async def confirm_check(check_id: int, actor_id: int, engine: object = None) -> dict
    # 返回 {"check_id", "missing", "surplus", "moved"}
    # 单 BEGIN IMMEDIATE 原子应用；任一项失败 → 整体回滚（session 保持 open）
    # 异常：CheckConflictError(409) / ComplianceError(422 由路由映射)
async def recover_missing(pipe_id: str, actor_id: int, engine: object = None) -> dict
    # 返回 {"pipe_id", "location"}；非 missing → CheckConflictError
```

### Steps

**Step 1 — check_service.py 追加**（文件末尾；import 补 `from src.models import Transaction, TransactionItem`、`from src.modules.warehouse.services.tube_service import _insert_tube_with_compliance`、`from src.modules.warehouse.services.audit_service import _write_audit`）：

```python
async def confirm_check(
    check_id: int,
    actor_id: int,
    engine: object = None,
) -> dict:
    """Atomically apply all differences of an open check session.

    ONE ``BEGIN IMMEDIATE`` transaction: every item is applied (missing →
    status='missing' + used_count-1; moved → location=actual + used_count
    migration; surplus → stock-in via the compliance core). Any failure
    raises → the whole txn rolls back, the session stays ``open`` and no
    audit row is written (zero partial application, spec §4.3).
    """
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        sess = await _require_open(conn, check_id)
        location = sess["location"]
        rows = (
            await conn.execute(
                text("SELECT * FROM check_items WHERE check_id=:c ORDER BY id"),
                {"c": check_id},
            )
        ).all()
        items = [dict(r._mapping) for r in rows]
        missing_n = surplus_n = moved_n = 0
        now = _now()

        for it in items:
            if it["result"] == "ok":
                continue
            if it["result"] == "missing":
                missing_n += 1
                r = await conn.execute(
                    text(
                        "UPDATE tubes SET status='missing', updated_at=:n "
                        "WHERE pipe_id=:p AND location=:l AND status='in_stock' "
                        "AND deleted_at IS NULL AND quality_mark != 'scrapped'"
                    ),
                    {"p": it["pipe_id"], "l": location, "n": now},
                )
                if r.rowcount != 1:
                    await conn.rollback()
                    raise CheckConflictError(
                        f"pipe '{it['pipe_id']}' not in_stock at '{location}' — re-check"
                    )
                await conn.execute(
                    text(
                        "UPDATE locations SET used_count=used_count-1 "
                        "WHERE code=:l AND used_count>0"
                    ),
                    {"l": location},
                )
            elif it["result"] == "moved":
                moved_n += 1
                target = it["actual_location"]
                loc = (
                    await conn.execute(
                        text(
                            "SELECT 1 FROM locations "
                            "WHERE code=:c AND is_active=1"
                        ),
                        {"c": target},
                    )
                ).fetchone()
                if loc is None:
                    await conn.rollback()
                    raise CheckConflictError(
                        f"location '{target}' not found or inactive"
                    )
                r = await conn.execute(
                    text(
                        "UPDATE tubes SET location=:t, updated_at=:n "
                        "WHERE pipe_id=:p AND location=:l AND status='in_stock' "
                        "AND deleted_at IS NULL AND quality_mark != 'scrapped'"
                    ),
                    {"p": it["pipe_id"], "l": location, "t": target, "n": now},
                )
                if r.rowcount != 1:
                    await conn.rollback()
                    raise CheckConflictError(
                        f"pipe '{it['pipe_id']}' not in_stock at '{location}' — re-check"
                    )
                await conn.execute(
                    text(
                        "UPDATE locations SET used_count=used_count-1 "
                        "WHERE code=:l AND used_count>0"
                    ),
                    {"l": location},
                )
                await conn.execute(
                    text(
                        "UPDATE locations SET used_count=used_count+1 "
                        "WHERE code=:t"
                    ),
                    {"t": target},
                )
            elif it["result"] == "surplus":
                surplus_n += 1
                data = json.loads(it["surplus_data"])
                await _insert_tube_with_compliance(
                    conn,
                    type_=data["type"],
                    outer_diameter_mm=data["outer_diameter_mm"],
                    wall_thickness_mm=data["wall_thickness_mm"],
                    length_m=data["length_m"],
                    steel_grade=data["steel_grade"],
                    location=location,
                    operator_id=actor_id,
                    manufacturer=data["manufacturer"],
                    pipe_id=it["pipe_id"],
                    thread_type=data.get("thread_type"),
                    api_5ct_cert=data.get("api_5ct_cert"),
                    batch_no=data.get("batch_no"),
                    heat_no=data.get("heat_no"),
                    heat_treat_method=data.get("heat_treat_method"),
                    quality_mark=data.get("quality_mark", "normal"),
                    quality_note=data.get("quality_note"),
                    seamless_specs=data.get("seamless_specs"),
                    screen_specs=data.get("screen_specs"),
                    compliance_pass=None,  # 盘盈走合规（spec §4.3）
                )

        txn_res = await conn.execute(
            Transaction.__table__.insert().values(
                type="check",
                operator_id=actor_id,
                remark=f"check {check_id}",
                created_at=now,
            )
        )
        txn_id = txn_res.inserted_primary_key[0]
        for it in items:
            if it["result"] == "ok":
                continue
            src = location if it["result"] in ("missing", "moved") else None
            tgt = (
                it["actual_location"]
                if it["result"] == "moved"
                else (location if it["result"] == "surplus" else None)
            )
            await conn.execute(
                TransactionItem.__table__.insert().values(
                    transaction_id=txn_id,
                    pipe_id=it["pipe_id"],
                    source_location=src,
                    target_location=tgt,
                )
            )

        await _write_audit(
            conn,
            "check",
            f"check_id={check_id} location={location} "
            f"missing={missing_n} surplus={surplus_n} moved={moved_n}",
            actor_id,
        )

        await conn.execute(
            text(
                "UPDATE check_sessions SET status='confirmed', "
                "confirmed_by=:a, confirmed_at=:t WHERE id=:i"
            ),
            {"a": actor_id, "t": now, "i": check_id},
        )
        await conn.commit()
        return {
            "check_id": check_id,
            "missing": missing_n,
            "surplus": surplus_n,
            "moved": moved_n,
        }


async def recover_missing(
    pipe_id: str,
    actor_id: int,
    engine: object = None,
) -> dict:
    """Restore a missing (盘亏) tube back to in_stock."""
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        tube = (
            await conn.execute(
                text("SELECT * FROM tubes WHERE pipe_id=:p AND deleted_at IS NULL"),
                {"p": pipe_id},
            )
        ).fetchone()
        if tube is None:
            await conn.rollback()
            raise CheckConflictError(f"pipe '{pipe_id}' not found")
        tm = dict(tube._mapping)
        if tm["status"] != "missing":
            await conn.rollback()
            raise CheckConflictError(
                f"pipe '{pipe_id}' is not missing (status={tm['status']})"
            )
        r = await conn.execute(
            text(
                "UPDATE tubes SET status='in_stock', updated_at=:n "
                "WHERE pipe_id=:p AND status='missing' AND deleted_at IS NULL"
            ),
            {"p": pipe_id, "n": _now()},
        )
        if r.rowcount != 1:
            await conn.rollback()
            raise CheckConflictError(f"pipe '{pipe_id}' recovery failed")
        loc = tm["location"]
        await conn.execute(
            text(
                "UPDATE locations SET used_count=used_count+1 WHERE code=:l"
            ),
            {"l": loc},
        )
        txn_res = await conn.execute(
            Transaction.__table__.insert().values(
                type="check",
                operator_id=actor_id,
                remark="recover",
                created_at=_now(),
            )
        )
        txn_id = txn_res.inserted_primary_key[0]
        await conn.execute(
            TransactionItem.__table__.insert().values(
                transaction_id=txn_id,
                pipe_id=pipe_id,
                source_location=loc,
                target_location=None,
            )
        )
        await _write_audit(
            conn, "recover", f"pipe_id={pipe_id} location={loc}", actor_id
        )
        await conn.commit()
        return {"pipe_id": pipe_id, "location": loc}
```

**Task 3 修正（控制器授权，审查轮追加 — 预防 Task 2 同类缺陷）**：

confirm 与 recover 路由的全部 `TemplateResponse` 渲染点同样必须遵循 FIX-A 模式（`csrf_token` 入 ctx + `set_csrf_cookie(rendered, token)`）；confirm 的 409/422 错误路径渲染 report 页时 ctx 含 `"check"`（来自 get_check_detail，用 try/except 包裹保证非 500）——参照本计划 Task 2 修正块。

**Step 2 — tubes.py 替换 2 个 handler**（注意：confirm 路由已在 Task 2 以 501 桩形式存在（约第 766 行），必须**替换**桩而非新增路由——重复注册会让桩先命中；recover 路由为新增）：（import 修改：src.security 块补 `require_admin`——第 23-29 行 `from src.security import (generate_csrf_token, get_current_user, require_admin, require_csrf, require_operator, set_csrf_cookie)`；`ComplianceError` 已导入（第 12 行），无需补）：

```python
@router.post("/check/{check_id}/confirm", dependencies=[Depends(require_csrf)])
async def check_confirm(
    request: Request,
    check_id: int,
    csrf_token: str = Form(...),
    user: User = Depends(require_admin),
):
    try:
        result = await confirm_check(check_id=check_id, actor_id=user.id)
    except ComplianceError as e:
        return templates.TemplateResponse(
            request,
            "warehouse/check_report.html",
            {
                "request": request,
                "detail": await get_check_detail(check_id),
                "counts": _report_counts(await get_check_detail(check_id)),
                "user": user,
                "error": str(e),
            },
            status_code=422,
        )
    except CheckConflictError as e:
        return templates.TemplateResponse(
            request,
            "warehouse/check_report.html",
            {
                "request": request,
                "detail": await get_check_detail(check_id),
                "counts": _report_counts(await get_check_detail(check_id)),
                "user": user,
                "error": str(e),
            },
            status_code=409,
        )
    return RedirectResponse(url=f"/tubes/check/{check_id}/report", status_code=303)


@router.post("/recover/{pipe_id}", dependencies=[Depends(require_csrf)])
async def tube_recover(
    request: Request,
    pipe_id: str,
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
):
    try:
        await recover_missing(pipe_id=pipe_id, actor_id=user.id)
    except CheckConflictError as e:
        return templates.TemplateResponse(
            request,
            "warehouse/tubes_list.html",
            {"request": request, "error": str(e)},
            status_code=409,
        )
    return RedirectResponse(url="/tubes", status_code=303)
```

（`_report_counts` 辅助——把 check_report 路由里的计数逻辑提取为模块级函数，check_confirm 复用；或在 confirm handler 内联同样逻辑。**要求：与 check_report 路由完全一致的计数逻辑，可提取为 `def _report_counts(detail) -> dict` 供两处复用**。）

**Step 3 — tests/test_check.py 追加**（文件末尾，8 项）：

```python
# --- 14-17, 18b, 19, 20: confirm & recover -----------------------------------

def test_14_admin_confirm_empty_session(client):
    _as(1)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/confirm")
        assert r.status_code == 303, r.text[:200]
    s = _sessions()[-1]
    assert s.status == "confirmed" and s.confirmed_by == 1
    assert _audit_count() == 1
    assert _audit_rows()[0].action == "check"
    assert "missing=0 surplus=0 moved=0" in _audit_rows()[0].detail


def test_15_admin_confirm_mixed(client):
    _as(2)  # 标记用 operator
    async def run():
        async with client as c:
            cid = await _create_session(c)
            await _post(c, f"/tubes/check/{cid}/items",
                        {"pipe_id": TUBES[0][0], "result": "missing"})
            await _post(c, f"/tubes/check/{cid}/items",
                        {"pipe_id": TUBES[1][0], "result": "moved",
                         "actual_location": WH_B})
            await _post(c, f"/tubes/check/{cid}/surplus", _surplus_payload())
            return cid
    cid = asyncio.run(run())
    app.dependency_overrides.clear()
    _as(1)  # 确认用 admin
    async def run2():
        async with client as c:
            r = await _post(c, f"/tubes/check/{cid}/confirm")
        assert r.status_code == 303, r.text[:200]
    asyncio.run(run2())
    s = _sessions()[-1]
    assert s.status == "confirmed" and s.confirmed_by == 1
    # missing 生效
    t1 = _tube(TUBES[0][0])
    assert t1.status == "missing"
    # moved 生效
    t2 = _tube(TUBES[1][0])
    assert t2.location == WH_B and t2.status == "in_stock"
    # surplus 生效（compliance 通过）
    surplus = _items(s.id)[-1]
    st = _tube(surplus.pipe_id)
    assert st is not None and st.status == "in_stock"
    assert st.location == WH_A and st.compliance_pass is True
    # used_count：WH-A 3-1(missing)-1(moved)+1(surplus)=2；WH-B 0+1=1
    assert _location(WH_A).used_count == 2
    assert _location(WH_B).used_count == 1
    # transaction：1 条 type=check + 1 条 type=in（盘盈核心）
    txns = _txns()
    check_txns = [t for t in txns if t.type == "check"]
    in_txns = [t for t in txns if t.type == "in"]
    assert len(check_txns) == 1 and len(in_txns) == 1
    assert check_txns[0].operator_id == 1
    # 审计：1 条 check + 1 条 stock_in
    audits = _audit_rows()
    actions = [a.action for a in audits]
    assert actions.count("check") == 1 and actions.count("stock_in") == 1


def test_16_confirm_atomicity_compliance_fail(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            await _post(c, f"/tubes/check/{cid}/items",
                        {"pipe_id": TUBES[0][0], "result": "missing"})
            bad = _surplus_payload()
            bad["steel_grade"] = "X80"  # 无标准规格 → compliance 失败
            await _post(c, f"/tubes/check/{cid}/surplus", bad)
            return cid
    cid = asyncio.run(run())
    app.dependency_overrides.clear()
    _as(1)
    async def run2():
        async with client as c:
            r = await _post(c, f"/tubes/check/{cid}/confirm")
        assert r.status_code == 422, r.text[:200]
    asyncio.run(run2())
    # 零变化：missing 未生效、session 仍 open、零审计、used_count 不变
    s = _sessions()[-1]
    assert s.status == "open"
    assert _tube(TUBES[0][0]).status == "in_stock"
    assert _audit_count() == 0
    assert _location(WH_A).used_count == 3


def test_17_confirm_toctou_rollback(client):
    from src.modules.warehouse.services.tube_service import stock_out
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            await _post(c, f"/tubes/check/{cid}/items",
                        {"pipe_id": TUBES[0][0], "result": "missing"})
            return cid
    cid = asyncio.run(run())
    # 确认前并发出库（模拟 TOCTOU）
    async def _out():
        await stock_out(pipe_id=TUBES[0][0], operator_id=1, role="admin",
                        engine=test_engine)
    asyncio.run(_out())
    assert _tube(TUBES[0][0]).status == "out_stock"
    _as(1)
    async def run2():
        async with client as c:
            r = await _post(c, f"/tubes/check/{cid}/confirm")
        assert r.status_code == 409, r.text[:200]
    asyncio.run(run2())
    # 整体回滚：无审计、无状态变化（管已 out_stock 未被改回）
    assert _audit_count() == 0
    assert _tube(TUBES[0][0]).status == "out_stock"
    assert _sessions()[-1].status == "open"


def test_18b_confirm_cancelled_session_409(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            await _post(c, f"/tubes/check/{cid}/cancel")
            return cid
    cid = asyncio.run(run())
    app.dependency_overrides.clear()
    _as(1)
    async def run2():
        async with client as c:
            r = await _post(c, f"/tubes/check/{cid}/confirm")
        assert r.status_code == 409, r.text[:200]
    asyncio.run(run2())
    assert _sessions()[-1].status == "cancelled"
    assert _audit_count() == 0


def test_19_recover_missing_tube(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            await _post(c, f"/tubes/check/{cid}/items",
                        {"pipe_id": TUBES[0][0], "result": "missing"})
            return cid
    cid = asyncio.run(run())
    app.dependency_overrides.clear()
    _as(1)
    async def run2():
        async with client as c:
            await _post(c, f"/tubes/check/{cid}/confirm")
            r = await _post(c, f"/tubes/recover/{TUBES[0][0]}")
        assert r.status_code == 303, r.text[:200]
    asyncio.run(run2())
    t1 = _tube(TUBES[0][0])
    assert t1.status == "in_stock"
    assert _location(WH_A).used_count == 3
    audits = _audit_rows()
    assert audits[-1].action == "recover"
    assert f"pipe_id={TUBES[0][0]}" in audits[-1].detail
    txns = _txns()
    assert any(t.type == "check" and t.remark == "recover" for t in txns)


def test_20_recover_non_missing_409(client):
    _as(2)
    async def run():
        async with client as c:
            r = await _post(c, "/tubes/recover/SP-L80-88.9x8.00-aaaac001")
        assert r.status_code == 409, r.text[:200]
    asyncio.run(run())
    assert _tube(TUBES[0][0]).status == "in_stock"
    assert _audit_count() == 0


def test_20b_chain_verifies_after_mixed_flow(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            await _post(c, f"/tubes/check/{cid}/items",
                        {"pipe_id": TUBES[0][0], "result": "missing"})
            await _post(c, f"/tubes/check/{cid}/items",
                        {"pipe_id": TUBES[1][0], "result": "moved",
                         "actual_location": WH_B})
            await _post(c, f"/tubes/check/{cid}/surplus", _surplus_payload())
            await _post(c, f"/tubes/check/{cid}/confirm")
            await _post(c, f"/tubes/recover/{TUBES[0][0]}")
    app.dependency_overrides.clear()
    _as(1)
    async def run2():
        async with client as c:
            v = await _post(c, "/audit/verify", {})
        assert v.status_code == 200, v.text[:400]
        assert "哈希链完整" in v.text
    asyncio.run(run())
    asyncio.run(run2())
```

**注意**：test_20b 是测试 20 的链校验扩展——编号对应 spec §7 测试 20（链验证）。测试 15/20b 中 operator 标记后清空 overrides 再切 admin（`app.dependency_overrides.clear()` + `_as(1)`）。

**Step 4 — TDD 验证**：

```bash
.venv/bin/python -m pytest tests/test_check.py -v
# RED：测试 14-17、18b、19、20、20b 失败（confirm/recover 路由 404），1-13/18a 仍绿
# 实现后：全绿（22 passed）
.venv/bin/python -m pytest -m "not slow"
# 全量回归：137 passed + 3 deselected
```

**预期输出**：RED 仅新增 8 项失败 → 全绿 22 passed；全量 **137 passed + 3 deselected**（最终功能基线）。

---

