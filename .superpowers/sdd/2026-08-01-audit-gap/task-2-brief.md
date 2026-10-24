### Task 2: standards_service.py 新建 + 路由瘦身

**Files:**
- Create: `src/modules/warehouse/services/standards_service.py`
- Modify: `src/modules/warehouse/routers/standards.py`
- Modify: `tests/test_admin_audit.py`（追加测试 9-12）

**Interfaces:**
- Consumes: `_write_audit`、`begin_immediate`（audit_service）；`db.engine`。
- Produces: `create_standard(steel_grade, outer_diameter_mm, min_wall_thickness_mm, max_wall_thickness_mm, engine=None, *, actor_id) -> None`；`update_standard(spec_id, steel_grade, outer_diameter_mm, min_wall_thickness_mm, max_wall_thickness_mm, engine=None, *, actor_id) -> None`；`delete_standard(spec_id, engine=None, *, actor_id) -> None`；`StandardConflictError`。路由 handler 重命名为 `create_standard_route` / `update_standard_route` / `delete_standard_route`（避免与 service 同名冲突）。

- [ ] **Step 1: 追加失败测试（测试 9-12）**

在 `tests/test_admin_audit.py` 文件末尾追加：

```python
# --- standards ops audit ---------------------------------------------------


def test_create_standard_writes_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/standards", {
                "steel_grade": "N80",
                "outer_diameter_mm": "114.3",
                "min_wall_thickness_mm": "7.0",
                "max_wall_thickness_mm": "11.0",
            })
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "create_standard"
    assert "grade=N80" in row.detail
    assert "od=114.3" in row.detail
    assert "min_wt=7.0" in row.detail
    assert "max_wt=11.0" in row.detail
    assert row.user_id == 1


def test_update_standard_writes_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/standards/1", {
                "steel_grade": "L80",
                "outer_diameter_mm": "90.0",
                "min_wall_thickness_mm": "6.0",
                "max_wall_thickness_mm": "10.0",
            })
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "update_standard"
    assert "od=90.0" in row.detail
    assert row.user_id == 1


def test_delete_standard_writes_audit_with_snapshot(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/standards/1/delete", {})
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "delete_standard"
    assert "grade=L80" in row.detail
    assert "od=88.9" in row.detail
    assert "min_wt=6.0" in row.detail
    assert "max_wt=10.0" in row.detail
    assert row.user_id == 1
    with SyncSession() as s:
        assert s.query(StandardSpec).filter(StandardSpec.id == 1).first() is None


def test_duplicate_grade_no_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/standards", {
                "steel_grade": "L80",
                "outer_diameter_mm": "88.9",
                "min_wall_thickness_mm": "6.0",
                "max_wall_thickness_mm": "10.0",
            })
        assert r.status_code == 409, r.text[:400]
    asyncio.run(run())
    assert _audit_count() == 0
```

- [ ] **Step 2: 运行确认测试失败**

Run: `.venv/bin/python -m pytest tests/test_admin_audit.py -k "standard" -v`
Expected: **3 failed, 1 passed**——FAIL（`assert row is not None`，当前无审计写入）：`test_create_standard_writes_audit`、`test_update_standard_writes_audit`、`test_delete_standard_writes_audit_with_snapshot`；PASS（零审计守卫在实现前即成立）：`test_duplicate_grade_no_audit`。

- [ ] **Step 3: 创建 standards_service.py**

新建 `src/modules/warehouse/services/standards_service.py`，完整内容：

```python
"""API 5CT standard spec service — create, update, delete with audit.

All operations use ``raw conn + begin_immediate`` and write the audit
entry inside the same transaction (project convention #13). The deleted
row's values are snapshotted into the audit detail before the hard delete
so the audit chain remains the only record of removed reference data.
"""

from __future__ import annotations

from sqlalchemy import select
from sqlalchemy.exc import IntegrityError

from src import database as db
from src.models import StandardSpec
from src.modules.warehouse.services.audit_service import (
    _write_audit,
    begin_immediate,
)


class StandardConflictError(Exception):
    """A StandardSpec with the same grade already exists."""


def _detail(grade: str, od: float, min_wt: float, max_wt: float) -> str:
    return (
        f"grade={grade} od={od} min_wt={min_wt} max_wt={max_wt}"
    )


async def create_standard(
    steel_grade: str,
    outer_diameter_mm: float,
    min_wall_thickness_mm: float,
    max_wall_thickness_mm: float,
    engine: object = None,
    *,
    actor_id: int,
) -> None:
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        grade = steel_grade.strip()
        try:
            await conn.execute(
                StandardSpec.__table__.insert().values(
                    grade=grade,
                    outer_diameter_mm=outer_diameter_mm,
                    min_wall_thickness_mm=min_wall_thickness_mm,
                    max_wall_thickness_mm=max_wall_thickness_mm,
                )
            )
        except IntegrityError:
            await conn.rollback()
            raise StandardConflictError(
                f"grade '{grade}' already exists"
            ) from None
        await _write_audit(
            conn, "create_standard",
            _detail(grade, outer_diameter_mm, min_wall_thickness_mm,
                    max_wall_thickness_mm),
            user_id=actor_id,
        )
        await conn.commit()


async def update_standard(
    spec_id: int,
    steel_grade: str,
    outer_diameter_mm: float,
    min_wall_thickness_mm: float,
    max_wall_thickness_mm: float,
    engine: object = None,
    *,
    actor_id: int,
) -> None:
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        row = (
            await conn.execute(
                select(StandardSpec.id).where(StandardSpec.id == spec_id)
            )
        ).fetchone()
        if row is None:
            await conn.rollback()
            raise ValueError(f"standard spec {spec_id} not found")
        grade = steel_grade.strip()
        try:
            await conn.execute(
                StandardSpec.__table__.update()
                .where(StandardSpec.id == spec_id)
                .values(
                    grade=grade,
                    outer_diameter_mm=outer_diameter_mm,
                    min_wall_thickness_mm=min_wall_thickness_mm,
                    max_wall_thickness_mm=max_wall_thickness_mm,
                )
            )
        except IntegrityError:
            await conn.rollback()
            raise StandardConflictError(
                f"grade '{grade}' already exists"
            ) from None
        await _write_audit(
            conn, "update_standard",
            _detail(grade, outer_diameter_mm, min_wall_thickness_mm,
                    max_wall_thickness_mm),
            user_id=actor_id,
        )
        await conn.commit()


async def delete_standard(
    spec_id: int,
    engine: object = None,
    *,
    actor_id: int,
) -> None:
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        row = (
            await conn.execute(
                select(
                    StandardSpec.grade,
                    StandardSpec.outer_diameter_mm,
                    StandardSpec.min_wall_thickness_mm,
                    StandardSpec.max_wall_thickness_mm,
                ).where(StandardSpec.id == spec_id)
            )
        ).fetchone()
        if row is None:
            await conn.rollback()
            raise ValueError(f"standard spec {spec_id} not found")
        await conn.execute(
            StandardSpec.__table__.delete().where(StandardSpec.id == spec_id)
        )
        await _write_audit(
            conn, "delete_standard",
            _detail(row.grade, row.outer_diameter_mm,
                    row.min_wall_thickness_mm, row.max_wall_thickness_mm),
            user_id=actor_id,
        )
        await conn.commit()
```

> 注意：原始 async conn 上 `select(StandardSpec)` 只返回主键（项目已知坑），故存在性检查用 `select(StandardSpec.id)`、删除快照用**显式列 select + `fetchone()`**（`row.grade` 等为 Row 具名访问，可靠）。

- [ ] **Step 4: 瘦身 standards 路由**

修改 `src/modules/warehouse/routers/standards.py`：

(a) 导入追加：

```python
from src.modules.warehouse.services.standards_service import (
    StandardConflictError,
    create_standard,
    delete_standard,
    update_standard,
)
```

(b) `create_standard` handler 改名为 `create_standard_route`，函数体改为：

```python
@router.post("/standards")
async def create_standard_route(
    request: Request,
    steel_grade: str = Form(...),
    outer_diameter_mm: float = Form(...),
    min_wall_thickness_mm: float = Form(...),
    max_wall_thickness_mm: float = Form(...),
    csrf_token: str = Form(...),  # noqa: ARG001
    user=Depends(get_current_user),
    _admin: User = Depends(require_admin),
    _: None = Depends(require_csrf),
):
    """Insert a new StandardSpec (audited)."""
    try:
        await create_standard(
            steel_grade=steel_grade,
            outer_diameter_mm=outer_diameter_mm,
            min_wall_thickness_mm=min_wall_thickness_mm,
            max_wall_thickness_mm=max_wall_thickness_mm,
            actor_id=user.id,
        )
    except StandardConflictError as e:
        csrf = request.cookies.get("csrf_token") or generate_csrf_token()
        rendered = templates.TemplateResponse(
            "standards/list.html",
            {
                "request": request,
                "user": user,
                "specs": [],
                "page": 1,
                "total_pages": 1,
                "total": 0,
                "grade_filter": None,
                "csrf_token": csrf,
                "error": str(e),
                "ok": None,
            },
            status_code=409,
        )
        set_csrf_cookie(rendered, csrf)
        return rendered

    return RedirectResponse(url="/standards", status_code=303)
```

（原 handler 的 `session: AsyncSession = Depends(get_session)` 参数移除，`IntegrityError` import 若无其他使用则一并移除；`error` 文案从固定「该钢级已存在」改为 `str(e)`，语义不变。）

(c) `update_standard` handler 改名为 `update_standard_route`，函数体改为：

```python
@router.post("/standards/{spec_id}")
async def update_standard_route(
    spec_id: int,
    steel_grade: str = Form(...),
    outer_diameter_mm: float = Form(...),
    min_wall_thickness_mm: float = Form(...),
    max_wall_thickness_mm: float = Form(...),
    csrf_token: str = Form(...),  # noqa: ARG001
    user=Depends(get_current_user),
    _admin: User = Depends(require_admin),
    _: None = Depends(require_csrf),
):
    try:
        await update_standard(
            spec_id=spec_id,
            steel_grade=steel_grade,
            outer_diameter_mm=outer_diameter_mm,
            min_wall_thickness_mm=min_wall_thickness_mm,
            max_wall_thickness_mm=max_wall_thickness_mm,
            actor_id=user.id,
        )
    except StandardConflictError:
        return JSONResponse(status_code=409, content={"detail": "该钢级已存在"})
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e)) from e

    return RedirectResponse(url="/standards", status_code=303)
```

（原 `session.get(StandardSpec, spec_id)` 的 404 检查由 service 的 `ValueError` 承接；`session` 参数移除。）

(d) `delete_standard` handler 改名为 `delete_standard_route`，函数体改为：

```python
@router.post("/standards/{spec_id}/delete")
async def delete_standard_route(
    spec_id: int,
    csrf_token: str = Form(...),  # noqa: ARG001
    user=Depends(get_current_user),
    _admin: User = Depends(require_admin),
    _: None = Depends(require_csrf),
):
    """Hard delete — pre-delete values snapshotted into the audit detail."""
    try:
        await delete_standard(spec_id=spec_id, actor_id=user.id)
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e)) from e

    return RedirectResponse(url="/standards", status_code=303)
```

（移除原 `session` 参数与 `IntegrityError` 无关逻辑；`func`/`select` 等 import 仅在只读 handler 使用，保留。）

- [ ] **Step 5: 运行测试确认通过**

Run: `.venv/bin/python -m pytest tests/test_admin_audit.py -v`
Expected: 12 passed。

- [ ] **Step 6: 既有 standards 测试回归**

Run: `.venv/bin/python -m pytest tests/test_routes_protection.py tests/test_tubes_routes.py -q`
Expected: 全通过（standards 相关 403/409 行为由这两个文件覆盖）。

- [ ] **Step 7: 全量回归**

Run: `.venv/bin/python -m pytest -m "not slow" -q`
Expected: 108 passed（104 + 4），3 deselected。

---

