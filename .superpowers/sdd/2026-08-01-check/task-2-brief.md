## Task 2: check_service 骨架 + 8 路由 + 3 模板 + 测试 1-13/18a

### Files
```
新 src/modules/warehouse/services/check_service.py      # create/mark/surplus/delete/get/list/cancel + detail
改 src/modules/warehouse/routers/tubes.py               # 8 路由（无 confirm/recover）
新 src/templates/warehouse/check_list.html
新 src/templates/warehouse/check_detail.html
新 src/templates/warehouse/check_report.html
新 tests/test_check.py                                  # 测试 1-13 + 18a（14 项）
```

### Interfaces

```python
async def create_check(location: str, operator_id: int, remark: str | None = None, engine: object = None) -> dict
async def get_check(check_id: int, engine: object = None) -> dict            # 不存在 → CheckConflictError
async def list_checks(engine: object = None) -> list[dict]
async def get_check_detail(check_id: int, engine: object = None) -> dict     # session + items + 库位在库管
async def mark_item(check_id: int, pipe_id: str, result: str, actual_location: str | None = None, actor_id: int, engine: object = None) -> dict
async def add_surplus(check_id: int, surplus_data: dict, actor_id: int, engine: object = None) -> dict
async def delete_surplus_item(check_id: int, item_id: int, actor_id: int, engine: object = None) -> None
async def cancel_check(check_id: int, actor_id: int, engine: object = None) -> None
```

- 全部 raw conn + begin_immediate（同 user_service）；返回 dict（`dict(row._mapping)`）
- `CheckConflictError(ConflictError)` 定义于此文件（`raise CheckConflictError(f"check session {check_id} is {status}")` 等）
- `mark_item` 校验：result ∈ CHECK_RESULTS；管存在且 `location==session.location` 且 `status=='in_stock'`；moved 需 `actual_location` 非空（否则 `raise ValueError("actual_location required for moved")` → 422）、≠ 盘点库位、目标库位存在且 active；upsert（同 check_id+pipe_id 更新，surplus_data 置 NULL）
- `add_surplus`：`_generate_pipe_id(type_, grade, od, wt)` 生成预览 pipe_id；tubes 中已存在 → CheckConflictError；`surplus_data=json.dumps(..., ensure_ascii=False)`；**仅格式校验，compliance 留到确认**
- `delete_surplus_item`：仅 result=='surplus' 可删（否则 CheckConflictError）；非 open → CheckConflictError

### Steps

**Step 1 — check_service.py**（全文，注意与上面接口完全一致）：

```python
"""Inventory check (盘点) service — D11.

All functions use raw conn + ``begin_immediate`` (same pattern as
user_service). Check sessions are DB-persisted; ``confirm_check`` applies
all differences atomically in ONE ``BEGIN IMMEDIATE`` transaction
(spec §4.3) — any failure rolls the whole txn back and keeps the session
``open`` (zero partial application, zero audit on failure).
"""

from __future__ import annotations

import json
from datetime import datetime, timezone
from typing import Any

from sqlalchemy import text

from src import database as db
from src.models import CheckItem, CheckSession
from src.modules.warehouse.services.audit_service import (
    ConflictError,
    begin_immediate,
)
from src.modules.warehouse.services.tube_service import _generate_pipe_id


class CheckConflictError(ConflictError):
    """409 for check-flow state conflicts (non-open session, bad marks)."""


CHECK_STATUSES = ("open", "confirmed", "cancelled")
CHECK_RESULTS = ("ok", "missing", "moved", "surplus")


def _now() -> datetime:
    return datetime.now(timezone.utc)


async def _load_session(conn, check_id: int) -> dict:
    row = (
        await conn.execute(
            text("SELECT * FROM check_sessions WHERE id=:i"), {"i": check_id}
        )
    ).fetchone()
    if row is None:
        raise CheckConflictError(f"check session {check_id} not found")
    return dict(row._mapping)


async def _require_open(conn, check_id: int) -> dict:
    sess = await _load_session(conn, check_id)
    if sess["status"] != "open":
        raise CheckConflictError(
            f"check session {check_id} is {sess['status']} (not open)"
        )
    return sess


async def create_check(
    location: str,
    operator_id: int,
    remark: str | None = None,
    engine: object = None,
) -> dict:
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        loc = (
            await conn.execute(
                text("SELECT 1 FROM locations WHERE code=:c AND is_active=1"),
                {"c": location},
            )
        ).fetchone()
        if loc is None:
            await conn.rollback()
            raise CheckConflictError(f"location '{location}' not found or inactive")
        res = await conn.execute(
            CheckSession.__table__.insert().values(
                location=location,
                status="open",
                created_by=operator_id,
                remark=remark,
                created_at=_now(),
            )
        )
        check_id = res.inserted_primary_key[0]
        await conn.commit()
        return {"id": check_id, "location": location}


async def get_check(check_id: int, engine: object = None) -> dict:
    eng = engine or db.engine
    async with eng.connect() as conn:
        return await _load_session(conn, check_id)


async def list_checks(engine: object = None) -> list[dict]:
    eng = engine or db.engine
    async with eng.connect() as conn:
        rows = (
            await conn.execute(
                text("SELECT * FROM check_sessions ORDER BY id DESC")
            )
        ).all()
        return [dict(r._mapping) for r in rows]


async def get_check_detail(check_id: int, engine: object = None) -> dict:
    eng = engine or db.engine
    async with eng.connect() as conn:
        sess = await _load_session(conn, check_id)
        items = (
            await conn.execute(
                text(
                    "SELECT * FROM check_items WHERE check_id=:c ORDER BY id"
                ),
                {"c": check_id},
            )
        ).all()
        tubes = (
            await conn.execute(
                text(
                    "SELECT pipe_id, steel_grade, outer_diameter_mm, "
                    "wall_thickness_mm, length_m, quality_mark, location "
                    "FROM tubes WHERE location=:l AND status='in_stock' "
                    "AND deleted_at IS NULL ORDER BY pipe_id"
                ),
                {"l": sess["location"]},
            )
        ).all()
        locations = (
            await conn.execute(
                text(
                    "SELECT code FROM locations WHERE is_active=1 ORDER BY code"
                )
            )
        ).all()
        return {
            "session": sess,
            "items": [dict(r._mapping) for r in items],
            "tubes": [dict(r._mapping) for r in tubes],
            "locations": [r._mapping["code"] for r in locations],
        }


async def mark_item(
    check_id: int,
    pipe_id: str,
    result: str,
    actual_location: str | None = None,
    actor_id: int = 0,
    engine: object = None,
) -> dict:
    if result not in CHECK_RESULTS:
        raise CheckConflictError(f"invalid result '{result}'")
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        sess = await _require_open(conn, check_id)
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
        if tm["location"] != sess["location"] or tm["status"] != "in_stock":
            await conn.rollback()
            raise CheckConflictError(
                f"pipe '{pipe_id}' not in_stock at '{sess['location']}'"
            )
        act_loc = actual_location.strip() if actual_location else None
        if result == "moved":
            if not act_loc:
                raise ValueError("actual_location required for moved")
            if act_loc == sess["location"]:
                await conn.rollback()
                raise CheckConflictError(
                    "actual_location must differ from session location"
                )
            loc = (
                await conn.execute(
                    text("SELECT 1 FROM locations WHERE code=:c AND is_active=1"),
                    {"c": act_loc},
                )
            ).fetchone()
            if loc is None:
                await conn.rollback()
                raise CheckConflictError(
                    f"location '{act_loc}' not found or inactive"
                )
        existing = (
            await conn.execute(
                text(
                    "SELECT id FROM check_items "
                    "WHERE check_id=:c AND pipe_id=:p"
                ),
                {"c": check_id, "p": pipe_id},
            )
        ).fetchone()
        if existing is not None:
            await conn.execute(
                CheckItem.__table__.update()
                .where(CheckItem.__table__.c.id == existing[0])
                .values(
                    result=result,
                    actual_location=act_loc,
                    surplus_data=None,
                )
            )
        else:
            await conn.execute(
                CheckItem.__table__.insert().values(
                    check_id=check_id,
                    pipe_id=pipe_id,
                    result=result,
                    actual_location=act_loc,
                    created_at=_now(),
                )
            )
        await conn.commit()
        return {"pipe_id": pipe_id, "result": result}


async def add_surplus(
    check_id: int,
    surplus_data: dict,
    actor_id: int = 0,
    engine: object = None,
) -> dict:
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        await _require_open(conn, check_id)
        pipe_type = surplus_data["type"]
        pipe_id = _generate_pipe_id(
            pipe_type,
            surplus_data["steel_grade"],
            surplus_data["outer_diameter_mm"],
            surplus_data["wall_thickness_mm"],
        )
        existing = (
            await conn.execute(
                text(
                    "SELECT 1 FROM tubes "
                    "WHERE pipe_id=:p AND deleted_at IS NULL"
                ),
                {"p": pipe_id},
            )
        ).fetchone()
        if existing is not None:
            await conn.rollback()
            raise CheckConflictError(f"pipe_id '{pipe_id}' already exists")
        res = await conn.execute(
            CheckItem.__table__.insert().values(
                check_id=check_id,
                pipe_id=pipe_id,
                result="surplus",
                surplus_data=json.dumps(surplus_data, ensure_ascii=False),
                created_at=_now(),
            )
        )
        await conn.commit()
        return {"pipe_id": pipe_id, "item_id": res.inserted_primary_key[0]}


async def delete_surplus_item(
    check_id: int,
    item_id: int,
    actor_id: int = 0,
    engine: object = None,
) -> None:
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        await _require_open(conn, check_id)
        item = (
            await conn.execute(
                text(
                    "SELECT * FROM check_items WHERE id=:i AND check_id=:c"
                ),
                {"i": item_id, "c": check_id},
            )
        ).fetchone()
        if item is None:
            await conn.rollback()
            raise CheckConflictError(f"check item {item_id} not found")
        if dict(item._mapping)["result"] != "surplus":
            await conn.rollback()
            raise CheckConflictError("only surplus items can be deleted")
        await conn.execute(
            text("DELETE FROM check_items WHERE id=:i"), {"i": item_id}
        )
        await conn.commit()


async def cancel_check(
    check_id: int,
    actor_id: int = 0,
    engine: object = None,
) -> None:
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        res = await conn.execute(
            text(
                "UPDATE check_sessions SET status='cancelled' "
                "WHERE id=:i AND status='open'"
            ),
            {"i": check_id},
        )
        if res.rowcount != 1:
            await conn.rollback()
            raise CheckConflictError(
                f"check session {check_id} cannot be cancelled (not open)"
            )
        await conn.commit()
```

**Step 2 — tubes.py 追加 8 路由**（import 修改：第 5 行 `from sqlalchemy import func, select` → `from sqlalchemy import func, select, text`；新增 `from src.modules.warehouse.services.check_service import CheckConflictError, add_surplus, cancel_check, create_check, delete_surplus_item, get_check_detail, list_checks, mark_item`；`Location` 已导入（第 9 行）；`require_admin` 本任务**不**需要（确认路由在 Task 3））：

```python
# ---------------------------------------------------------------------------
# Inventory check (盘点, D11)
# ---------------------------------------------------------------------------


@router.get("/check", dependencies=[Depends(require_operator)])
async def check_list(
    request: Request,
    session: AsyncSession = Depends(get_session),
    _: None = Depends(require_operator),
):
    checks = await list_checks()
    locations = (
        await session.execute(
            text("SELECT code FROM locations WHERE is_active=1 ORDER BY code")
        )
    ).scalars().all()
    return templates.TemplateResponse(
        request,
        "warehouse/check_list.html",
        {"request": request, "checks": checks, "locations": locations},
    )


@router.post("/check", dependencies=[Depends(require_csrf)])
async def check_create(
    request: Request,
    location: str = Form(...),
    remark: str = Form(""),
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
):
    try:
        created = await create_check(location=location, operator_id=user.id, remark=remark or None)
    except CheckConflictError as e:
        checks = await list_checks()
        return templates.TemplateResponse(
            request,
            "warehouse/check_list.html",
            {
                "request": request,
                "checks": checks,
                "locations": await _active_location_codes(),
                "error": str(e),
                "form": {"location": location, "remark": remark},
            },
            status_code=409,
        )
    return RedirectResponse(url=f"/tubes/check/{created['id']}", status_code=303)


@router.get("/check/{check_id}", dependencies=[Depends(require_operator)])
async def check_detail(
    request: Request,
    check_id: int,
    user: User = Depends(get_current_user),
):
    try:
        detail = await get_check_detail(check_id)
    except CheckConflictError as e:
        return templates.TemplateResponse(
            request,
            "warehouse/check_list.html",
            {"request": request, "checks": await list_checks(), "error": str(e)},
            status_code=409,
        )
    return templates.TemplateResponse(
        request,
        "warehouse/check_detail.html",
        {"request": request, "detail": detail, "user": user},
    )


@router.post("/check/{check_id}/items", dependencies=[Depends(require_csrf)])
async def check_mark_item(
    request: Request,
    check_id: int,
    pipe_id: str = Form(...),
    result: str = Form(...),
    actual_location: str = Form(""),
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
):
    try:
        await mark_item(
            check_id=check_id,
            pipe_id=pipe_id,
            result=result,
            actual_location=actual_location or None,
            actor_id=user.id,
        )
    except ValueError as e:
        return templates.TemplateResponse(
            request,
            "warehouse/check_detail.html",
            {
                "request": request,
                "detail": await get_check_detail(check_id),
                "user": user,
                "error": str(e),
            },
            status_code=422,
        )
    except CheckConflictError as e:
        return templates.TemplateResponse(
            request,
            "warehouse/check_detail.html",
            {
                "request": request,
                "detail": await get_check_detail(check_id),
                "user": user,
                "error": str(e),
            },
            status_code=409,
        )
    return RedirectResponse(url=f"/tubes/check/{check_id}", status_code=303)


@router.post("/check/{check_id}/surplus", dependencies=[Depends(require_csrf)])
async def check_add_surplus(
    request: Request,
    check_id: int,
    type: str = Form(..., alias="type"),
    outer_diameter_mm: float = Form(...),
    wall_thickness_mm: float = Form(...),
    length_m: float = Form(...),
    steel_grade: str = Form(...),
    manufacturer: str = Form(...),
    thread_type: str = Form(""),
    api_5ct_cert: str = Form(""),
    batch_no: str = Form(""),
    heat_no: str = Form(""),
    heat_treat_method: str = Form(""),
    quality_mark: str = Form("normal"),
    quality_note: str = Form(""),
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
):
    surplus_data = {
        "type": type,
        "outer_diameter_mm": outer_diameter_mm,
        "wall_thickness_mm": wall_thickness_mm,
        "length_m": length_m,
        "steel_grade": steel_grade,
        "manufacturer": manufacturer,
        "thread_type": thread_type or None,
        "api_5ct_cert": api_5ct_cert or None,
        "batch_no": batch_no or None,
        "heat_no": heat_no or None,
        "heat_treat_method": heat_treat_method or None,
        "quality_mark": quality_mark,
        "quality_note": quality_note or None,
    }
    try:
        await add_surplus(check_id=check_id, surplus_data=surplus_data, actor_id=user.id)
    except (ValueError, KeyError) as e:
        return templates.TemplateResponse(
            request,
            "warehouse/check_detail.html",
            {
                "request": request,
                "detail": await get_check_detail(check_id),
                "user": user,
                "error": f"盘盈信息不完整: {e}",
            },
            status_code=422,
        )
    except CheckConflictError as e:
        return templates.TemplateResponse(
            request,
            "warehouse/check_detail.html",
            {
                "request": request,
                "detail": await get_check_detail(check_id),
                "user": user,
                "error": str(e),
            },
            status_code=409,
        )
    return RedirectResponse(url=f"/tubes/check/{check_id}", status_code=303)


@router.post("/check/{check_id}/surplus/{item_id}/delete", dependencies=[Depends(require_csrf)])
async def check_delete_surplus(
    request: Request,
    check_id: int,
    item_id: int,
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
):
    try:
        await delete_surplus_item(check_id=check_id, item_id=item_id, actor_id=user.id)
    except CheckConflictError as e:
        return templates.TemplateResponse(
            request,
            "warehouse/check_detail.html",
            {
                "request": request,
                "detail": await get_check_detail(check_id),
                "user": user,
                "error": str(e),
            },
            status_code=409,
        )
    return RedirectResponse(url=f"/tubes/check/{check_id}", status_code=303)


@router.get("/check/{check_id}/report", dependencies=[Depends(require_operator)])
async def check_report(
    request: Request,
    check_id: int,
    user: User = Depends(get_current_user),
):
    try:
        detail = await get_check_detail(check_id)
    except CheckConflictError as e:
        return templates.TemplateResponse(
            request,
            "warehouse/check_list.html",
            {"request": request, "checks": await list_checks(), "error": str(e)},
            status_code=409,
        )
    counts = {"ok": 0, "missing": 0, "moved": 0, "surplus": 0}
    for it in detail["items"]:
        counts[it["result"]] = counts.get(it["result"], 0) + 1
    return templates.TemplateResponse(
        request,
        "warehouse/check_report.html",
        {"request": request, "detail": detail, "counts": counts, "user": user},
    )


@router.post("/check/{check_id}/cancel", dependencies=[Depends(require_csrf)])
async def check_cancel(
    request: Request,
    check_id: int,
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
):
    try:
        await cancel_check(check_id=check_id, actor_id=user.id)
    except CheckConflictError as e:
        return templates.TemplateResponse(
            request,
            "warehouse/check_detail.html",
            {
                "request": request,
                "detail": await get_check_detail(check_id),
                "user": user,
                "error": str(e),
            },
            status_code=409,
        )
    return RedirectResponse(url="/tubes/check", status_code=303)
```

（辅助：`_active_location_codes()` 定义于 tubes.py 模块级：）

```python
async def _active_location_codes(session: AsyncSession = None) -> list[str]:
    from src import database as db_mod
    if session is None:
        async with db_mod.async_session_factory() as s:
            rows = (await s.execute(text("SELECT code FROM locations WHERE is_active=1 ORDER BY code"))).scalars().all()
            return list(rows)
    rows = (await session.execute(text("SELECT code FROM locations WHERE is_active=1 ORDER BY code"))).scalars().all()
    return list(rows)
```

**Task 2 修正（控制器授权，审查轮追加 — 修正本计划自身缺陷，实现者照此执行）**：

**FIX-A（Critical，csrf）**：本任务全部 6 处 `templates.TemplateResponse(...)` 渲染点（check_list、check_detail 正常路径、check_report、check_detail 409、check_report 409、cancel 409）**必须**遵循既有模式（tubes.py 第 111-121 行 stock_out GET 同款）：
```python
token = request.cookies.get("csrf_token") or generate_csrf_token()
rendered = templates.TemplateResponse(
    "warehouse/check_list.html",
    {"request": request, "user": user, "checks": checks, "error": "...", "csrf_token": token},
)
set_csrf_cookie(rendered, token)
return rendered
```
（6 个渲染点全部加 `csrf_token` 入 ctx + `set_csrf_cookie(rendered, token)`。tubes.py 第 34/39 行已导入 `generate_csrf_token`/`set_csrf_cookie`，无需补导入。）

**FIX-B（Important a）**：check_detail 与 check_report 的 409 错误路径渲染 `check_list.html` 时，ctx 必须含 `"locations": await _active_location_codes()`（否则模板 `{% for loc in locations %}` 对未定义变量迭代抛 UndefinedError → 500）。

**FIX-C（Important b）**：mark/surplus/delete/cancel 的错误重渲染路径若调用 `get_check_detail(check_id)`，对**不存在的会话**该方法会再次抛 CheckConflictError → 必须用 try/except CheckConflictError 包裹：捕获后渲染 `check_list.html`（ctx 含 error + locations + csrf_token + set_csrf_cookie）。**任何错误路径都不允许 500**。

验证追加：GREEN 后手动补验——GET /tubes/check/99999 → 200 渲染列表 + 错误提示（非 500）；渲染 HTML 中 `<input type="hidden" name="csrf_token" value="` 后跟非空值（6 个渲染点抽查 2 处）。


**Step 3 — 3 个模板**：

`check_list.html`：

```html
{% extends "base.html" %}
{% block content %}
<div class="page-head">
  <h2>库存盘点</h2>
  <p class="text-muted">选择库位新建盘点单，逐根核对后由管理员确认。</p>
</div>

{% if error %}<div class="alert alert-danger">{{ error }}</div>{% endif %}

<div class="card mb-4">
  <div class="card-body">
    <form method="post" action="/tubes/check" class="row g-3 align-items-end">
      <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
      <div class="col-auto">
        <label class="form-label">库位</label>
        <select name="location" class="form-select" required>
          {% for loc in locations %}<option value="{{ loc }}">{{ loc }}</option>{% endfor %}
        </select>
      </div>
      <div class="col-auto">
        <label class="form-label">备注（可选）</label>
        <input type="text" name="remark" class="form-control" value="{{ form.remark if form else '' }}">
      </div>
      <div class="col-auto">
        <button type="submit" class="btn btn-accent">新建盘点单</button>
      </div>
    </form>
  </div>
</div>

<table class="table">
  <thead><tr><th>#</th><th>库位</th><th>状态</th><th>创建时间</th><th>操作</th></tr></thead>
  <tbody>
  {% for c in checks %}
    <tr>
      <td>{{ c.id }}</td>
      <td>{{ c.location }}</td>
      <td>{{ c.status }}</td>
      <td>{{ c.created_at }}</td>
      <td><a class="btn btn-sm btn-outline-primary" href="/tubes/check/{{ c.id }}">核对</a>
          <a class="btn btn-sm btn-outline-secondary" href="/tubes/check/{{ c.id }}/report">报告</a></td>
    </tr>
  {% else %}
    <tr><td colspan="5" class="text-muted">暂无盘点单</td></tr>
  {% endfor %}
  </tbody>
</table>
{% endblock %}
```

`check_detail.html`：

```html
{% extends "base.html" %}
{% block content %}
<div class="page-head">
  <h2>盘点核对 — {{ detail.session.location }}（#{{ detail.session.id }}）</h2>
  <p class="text-muted">状态：{{ detail.session.status }}</p>
  <a class="btn btn-sm btn-outline-secondary" href="/tubes/check/{{ detail.session.id }}/report">查看报告</a>
</div>

{% if error %}<div class="alert alert-danger">{{ error }}</div>{% endif %}

{% if detail.session.status == "open" %}
<h5 class="mt-4">在库管子</h5>
<table class="table table-sm">
  <thead><tr><th>pipe_id</th><th>规格</th><th>质量</th><th>核对结果</th><th>操作</th></tr></thead>
  <tbody>
  {% for t in detail.tubes %}
    {% set item = detail.items | selectattr("pipe_id", "equalto", t.pipe_id) | first %}
    <tr>
      <td>{{ t.pipe_id }}</td>
      <td>{{ t.steel_grade }} {{ t.outer_diameter_mm }}x{{ t.wall_thickness_mm }} {{ t.length_m }}m</td>
      <td>{{ t.quality_mark }}</td>
      <td>{% if item %}<span class="badge bg-info">{{ item.result }}</span>{% else %}<span class="text-muted">未核对</span>{% endif %}</td>
      <td class="d-flex gap-1">
        <form method="post" action="/tubes/check/{{ detail.session.id }}/items">
          <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
          <input type="hidden" name="pipe_id" value="{{ t.pipe_id }}">
          <input type="hidden" name="result" value="ok">
          <button class="btn btn-sm btn-outline-success">一致</button>
        </form>
        <form method="post" action="/tubes/check/{{ detail.session.id }}/items">
          <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
          <input type="hidden" name="pipe_id" value="{{ t.pipe_id }}">
          <input type="hidden" name="result" value="missing">
          <button class="btn btn-sm btn-outline-danger">缺失</button>
        </form>
        <form method="post" action="/tubes/check/{{ detail.session.id }}/items" class="d-flex gap-1">
          <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
          <input type="hidden" name="pipe_id" value="{{ t.pipe_id }}">
          <input type="hidden" name="result" value="moved">
          <select name="actual_location" class="form-select form-select-sm">
            {% for loc in detail.locations %}{% if loc != detail.session.location %}<option value="{{ loc }}">{{ loc }}</option>{% endif %}{% endfor %}
          </select>
          <button class="btn btn-sm btn-outline-warning">位置异常</button>
        </form>
      </td>
    </tr>
  {% else %}
    <tr><td colspan="5" class="text-muted">该库位无在库管子</td></tr>
  {% endfor %}
  </tbody>
</table>

<h5 class="mt-4">盘盈录入</h5>
<form method="post" action="/tubes/check/{{ detail.session.id }}/surplus" class="row g-3">
  <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
  <div class="col-2"><label class="form-label">类型</label>
    <select name="type" class="form-select"><option value="seamless_pipe">无缝</option><option value="screen_pipe">筛管</option></select></div>
  <div class="col-2"><label class="form-label">钢级</label><input name="steel_grade" class="form-control" required></div>
  <div class="col-2"><label class="form-label">外径 mm</label><input name="outer_diameter_mm" type="number" step="any" class="form-control" required></div>
  <div class="col-2"><label class="form-label">壁厚 mm</label><input name="wall_thickness_mm" type="number" step="any" class="form-control" required></div>
  <div class="col-2"><label class="form-label">长度 m</label><input name="length_m" type="number" step="any" class="form-control" required></div>
  <div class="col-2"><label class="form-label">生产厂</label><input name="manufacturer" class="form-control" required></div>
  <div class="col-2"><label class="form-label">螺纹</label><input name="thread_type" class="form-control"></div>
  <div class="col-2"><label class="form-label">API 5CT 证书</label><input name="api_5ct_cert" class="form-control"></div>
  <div class="col-2"><label class="form-label">批次</label><input name="batch_no" class="form-control"></div>
  <div class="col-2"><label class="form-label">炉号</label><input name="heat_no" class="form-control"></div>
  <div class="col-2"><label class="form-label">热处理</label><input name="heat_treat_method" class="form-control"></div>
  <div class="col-2"><label class="form-label">质量</label>
    <select name="quality_mark" class="form-select"><option value="normal">normal</option><option value="problematic">problematic</option></select></div>
  <div class="col-2"><label class="form-label">备注</label><input name="quality_note" class="form-control"></div>
  <div class="col-12"><button class="btn btn-accent">添加盘盈</button></div>
</form>
{% endif %}

<h5 class="mt-4">已记录项</h5>
<table class="table table-sm">
  <thead><tr><th>pipe_id</th><th>结果</th><th>实际库位</th><th></th></tr></thead>
  <tbody>
  {% for it in detail.items %}
    <tr>
      <td>{{ it.pipe_id }}</td>
      <td>{{ it.result }}</td>
      <td>{{ it.actual_location or "" }}</td>
      <td>
        {% if it.result == "surplus" and detail.session.status == "open" %}
        <form method="post" action="/tubes/check/{{ detail.session.id }}/surplus/{{ it.id }}/delete">
          <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
          <button class="btn btn-sm btn-outline-danger">删除</button>
        </form>
        {% endif %}
      </td>
    </tr>
  {% else %}
    <tr><td colspan="4" class="text-muted">暂无记录</td></tr>
  {% endfor %}
  </tbody>
</table>
{% endblock %}
```

`check_report.html`：

```html
{% extends "base.html" %}
{% block content %}
<div class="page-head">
  <h2>盘点报告 — {{ detail.session.location }}（#{{ detail.session.id }}）</h2>
  <p class="text-muted">状态：{{ detail.session.status }}</p>
</div>

{% if error %}<div class="alert alert-danger">{{ error }}</div>{% endif %}

<div class="row g-3 mb-4">
  <div class="col-3"><div class="card"><div class="card-body"><h4>{{ counts.ok }}</h4><div class="text-muted">一致</div></div></div></div>
  <div class="col-3"><div class="card"><div class="card-body"><h4>{{ counts.missing }}</h4><div class="text-muted">缺失</div></div></div></div>
  <div class="col-3"><div class="card"><div class="card-body"><h4>{{ counts.moved }}</h4><div class="text-muted">位置异常</div></div></div></div>
  <div class="col-3"><div class="card"><div class="card-body"><h4>{{ counts.surplus }}</h4><div class="text-muted">盘盈</div></div></div></div>
</div>

<table class="table table-sm">
  <thead><tr><th>pipe_id</th><th>结果</th><th>实际库位</th></tr></thead>
  <tbody>
  {% for it in detail.items %}
    <tr><td>{{ it.pipe_id }}</td><td>{{ it.result }}</td><td>{{ it.actual_location or "" }}</td></tr>
  {% endfor %}
  </tbody>
</table>

{% if detail.session.status == "open" %}
<div class="d-flex gap-2">
  {% if user.role == "admin" %}
  <form method="post" action="/tubes/check/{{ detail.session.id }}/confirm">
    <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
    <button class="btn btn-accent">管理员确认</button>
  </form>
  {% endif %}
  <form method="post" action="/tubes/check/{{ detail.session.id }}/cancel">
    <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
    <button class="btn btn-outline-danger">取消盘点单</button>
  </form>
</div>
{% endif %}
{% endblock %}
```

**Step 4 — tests/test_check.py**（全文；测试 1-13 + 18a）：

```python
"""Inventory check (盘点) tests — D11.

Covers the check lifecycle: create, mark (ok/missing/moved), surplus
add/delete, cancel, role matrix. Confirm/recover atomicity tests live in
the confirm task (appended later in this file).
"""

from __future__ import annotations

import json
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

TEST_DB_PATH = "data/test_check.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-secret-check")

from datetime import datetime, timezone  # noqa: E402

from httpx import ASGITransport, AsyncClient  # noqa: E402
import pytest  # noqa: E402
from sqlalchemy import create_engine as create_sync_engine  # noqa: E402
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine  # noqa: E402
from sqlalchemy.orm import sessionmaker as sync_sessionmaker  # noqa: E402

from rust_core import auth as rust_auth  # noqa: E402

rust_auth.init(os.environ["JWT_SECRET_KEY"])
from src import database as db  # noqa: E402
from src.main import app  # noqa: E402
from src.models import (  # noqa: E402
    AuditLog, Base, CheckItem, CheckSession, Location, StandardSpec,
    Transaction, TransactionItem, Tube, User,
)
from src.security import get_current_user, require_csrf  # noqa: E402

SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)

TUBES = [
    ("SP-L80-88.9x8.00-aaaac001", "normal"),
    ("SP-L80-88.9x8.00-aaaac002", "normal"),
    ("SP-L80-88.9x8.00-aaaac003", "normal"),
]
WH_A = "WH-A"
WH_B = "WH-B"


def _seed():
    Base.metadata.create_all(bind=sync_engine)
    with SyncSession() as s:
        for m in (AuditLog, TransactionItem, Transaction, CheckItem,
                  CheckSession, Tube, StandardSpec, Location, User):
            s.query(m).delete()
        s.commit()
        s.add(Location(code=WH_A, name="Warehouse A", used_count=3, is_active=True))
        s.add(Location(code=WH_B, name="Warehouse B", used_count=0, is_active=True))
        s.add(StandardSpec(
            grade="L80", outer_diameter_mm=88.9,
            min_wall_thickness_mm=6.0, max_wall_thickness_mm=10.0,
            length_tolerance_mm=0.5, source="API 5CT",
        ))
        s.add(User(id=1, username="admin", role="admin", display_name="Admin",
                   password_hash=rust_auth.hash_password("Pa$$w0rd"), active=True, token_version=0))
        s.add(User(id=2, username="operator", role="operator", display_name="Operator",
                   password_hash=rust_auth.hash_password("Pa$$w0rd"), active=True, token_version=0))
        s.add(User(id=3, username="viewer", role="viewer", display_name="Viewer",
                   password_hash=rust_auth.hash_password("Pa$$w0rd"), active=True, token_version=0))
        now = datetime.now(timezone.utc)
        for pid, qm in TUBES:
            s.add(Tube(
                pipe_id=pid, type="seamless_pipe",
                outer_diameter_mm=88.9, wall_thickness_mm=8.0, length_m=12.0,
                steel_grade="L80", location=WH_A, status="in_stock",
                quality_mark=qm, manufacturer="TestCo",
                stock_in_time=now, created_at=now, updated_at=now,
            ))
        s.commit()


def _reset_db():
    if os.path.exists(TEST_DB_PATH):
        os.remove(TEST_DB_PATH)
    for ext in ("-wal", "-shm"):
        p = TEST_DB_PATH + ext
        if os.path.exists(p):
            os.remove(p)


def _allow_csrf():
    async def _dep():
        return None
    return _dep


def _stub_user(user_id: int):
    async def _dep():
        with SyncSession() as s:
            return s.get(User, user_id)
    return _dep


def _audit_rows():
    with SyncSession() as s:
        return s.query(AuditLog).order_by(AuditLog.id).all()


def _audit_count():
    with SyncSession() as s:
        return s.query(AuditLog).count()


def _sessions():
    with SyncSession() as s:
        return s.query(CheckSession).order_by(CheckSession.id).all()


def _items(check_id: int):
    with SyncSession() as s:
        return s.query(CheckItem).filter(CheckItem.check_id == check_id).order_by(CheckItem.id).all()


def _tube(pipe_id: str):
    with SyncSession() as s:
        return s.query(Tube).filter(Tube.pipe_id == pipe_id).first()


def _location(code: str):
    with SyncSession() as s:
        return s.query(Location).filter(Location.code == code).first()


def _txns():
    with SyncSession() as s:
        return s.query(Transaction).order_by(Transaction.id).all()


@pytest.fixture
def client():
    transport = ASGITransport(app=app)
    return AsyncClient(transport=transport, base_url="http://test")


@pytest.fixture(autouse=True)
def fx():
    _reset_db()
    _seed()
    db.engine = test_engine
    db.async_session_factory = async_sessionmaker(
        test_engine, class_=AsyncSession, expire_on_commit=False,
    )
    yield
    app.dependency_overrides.clear()
    Base.metadata.drop_all(bind=sync_engine)
    sync_engine.dispose()


async def _post(client, url: str, data: dict | None = None, follow: bool = False):
    return await client.post(
        url,
        data={**(data or {}), "csrf_token": "test-csrf"},
        follow_redirects=follow,
    )


def _as(user_id: int):
    app.dependency_overrides[get_current_user] = _stub_user(user_id)
    app.dependency_overrides[require_csrf] = _allow_csrf()


async def _create_session(client, location: str = WH_A) -> int:
    r = await _post(client, "/tubes/check", {"location": location, "remark": "r1"})
    assert r.status_code == 303, r.text[:200]
    return _sessions()[-1].id


# --- 1-2: role matrix -------------------------------------------------------

def test_1_viewer_get_check_list_forbidden(client):
    _as(3)
    async def run():
        async with client as c:
            r = await c.get("/tubes/check")
        assert r.status_code == 403, r.text[:200]
    asyncio.run(run())


def test_2_viewer_post_check_create_forbidden(client):
    _as(3)
    async def run():
        async with client as c:
            r = await _post(c, "/tubes/check", {"location": WH_A})
        assert r.status_code == 403, r.text[:200]
    asyncio.run(run())


# --- 3: create --------------------------------------------------------------

def test_3_operator_create_check(client):
    _as(2)
    async def run():
        async with client as c:
            r = await _post(c, "/tubes/check", {"location": WH_A, "remark": "r1"})
        assert r.status_code == 303, r.text[:200]
    s = _sessions()
    assert len(s) == 1
    assert s[0].location == WH_A and s[0].status == "open" and s[0].created_by == 2


# --- 4-9: mark --------------------------------------------------------------

def test_4_mark_ok(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/items",
                            {"pipe_id": TUBES[0][0], "result": "ok"})
        assert r.status_code == 303, r.text[:200]
    items = _items(_sessions()[-1].id)
    assert len(items) == 1 and items[0].result == "ok"


def test_5_mark_missing(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/items",
                            {"pipe_id": TUBES[0][0], "result": "missing"})
        assert r.status_code == 303, r.text[:200]
    items = _items(_sessions()[-1].id)
    assert len(items) == 1 and items[0].result == "missing"


def test_6_mark_moved_without_location_422(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/items",
                            {"pipe_id": TUBES[0][0], "result": "moved",
                             "actual_location": ""})
        assert r.status_code == 422, r.text[:200]
    assert len(_items(_sessions()[-1].id)) == 0


def test_7_mark_moved_same_location_409(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/items",
                            {"pipe_id": TUBES[0][0], "result": "moved",
                             "actual_location": WH_A})
        assert r.status_code == 409, r.text[:200]
    assert len(_items(_sessions()[-1].id)) == 0


def test_8_mark_moved_valid(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/items",
                            {"pipe_id": TUBES[0][0], "result": "moved",
                             "actual_location": WH_B})
        assert r.status_code == 303, r.text[:200]
    items = _items(_sessions()[-1].id)
    assert len(items) == 1
    assert items[0].result == "moved" and items[0].actual_location == WH_B


def test_9_mark_upsert_updates_single_row(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            await _post(c, f"/tubes/check/{cid}/items",
                        {"pipe_id": TUBES[0][0], "result": "missing"})
            r = await _post(c, f"/tubes/check/{cid}/items",
                            {"pipe_id": TUBES[0][0], "result": "ok"})
        assert r.status_code == 303, r.text[:200]
    items = _items(_sessions()[-1].id)
    assert len(items) == 1 and items[0].result == "ok"


# --- 10-11: surplus ---------------------------------------------------------

def _surplus_payload():
    return {
        "type": "seamless_pipe", "outer_diameter_mm": 88.9,
        "wall_thickness_mm": 8.0, "length_m": 12.0, "steel_grade": "L80",
        "manufacturer": "TestCo", "quality_mark": "normal",
    }


def test_10_add_surplus_creates_item_with_preview_pipe_id(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/surplus", _surplus_payload())
        assert r.status_code == 303, r.text[:200]
    items = _items(_sessions()[-1].id)
    assert len(items) == 1
    it = items[0]
    assert it.result == "surplus"
    import re
    assert re.match(r"SP-L80-88\.9x8\.00-[0-9a-f]{8}$", it.pipe_id)
    data = json.loads(it.surplus_data)
    assert data["steel_grade"] == "L80" and data["outer_diameter_mm"] == 88.9


def test_11_delete_surplus_item(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            await _post(c, f"/tubes/check/{cid}/surplus", _surplus_payload())
            await _post(c, f"/tubes/check/{cid}/items",
                        {"pipe_id": TUBES[0][0], "result": "ok"})
            r = await _post(c, f"/tubes/check/{cid}/surplus/1/delete")
        assert r.status_code == 303, r.text[:200]
        r2 = await _post(c, f"/tubes/check/{cid}/surplus/2/delete")  # ok 项不可删
        assert r2.status_code == 409, r2.text[:200]
    items = _items(_sessions()[-1].id)
    assert len(items) == 1 and items[0].result == "ok"


# --- 12 + 18a: cancel / non-open --------------------------------------------

def test_18a_cancel_open_session(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/cancel")
        assert r.status_code == 303, r.text[:200]
    assert _sessions()[-1].status == "cancelled"


def test_12_mark_non_open_session_409(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            await _post(c, f"/tubes/check/{cid}/cancel")
            r = await _post(c, f"/tubes/check/{cid}/items",
                            {"pipe_id": TUBES[0][0], "result": "ok"})
        assert r.status_code == 409, r.text[:200]
    assert len(_items(_sessions()[-1].id)) == 0


# --- 13: operator cannot confirm ---------------------------------------------

def test_13_operator_confirm_forbidden(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/confirm")
        assert r.status_code == 403, r.text[:200]
    asyncio.run(run())
```

（顶部需补 `import asyncio` 与 `import re`——Step 4 文件顶部加 `import asyncio`；`import re` 在 test_10 内联导入亦可。**注意：文件顶部必须有 `import asyncio`**。）

**Step 5 — TDD 验证**：

```bash
.venv/bin/python -m pytest tests/test_check.py -v
# RED：全部失败（路由不存在 → 404/断言失败），预期 14 failed
# 实现后：
# GREEN：14 passed
.venv/bin/python -m pytest -m "not slow"
# 全量回归：129 passed + 3 deselected（115 + 14）
```

**预期输出**：RED 全失败 → 实现后 14 passed；全量 129 passed。

---

