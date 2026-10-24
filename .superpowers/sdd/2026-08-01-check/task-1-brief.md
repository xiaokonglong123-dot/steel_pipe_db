## Task 1: 迁移 + ORM 模型 + stock_in 核心提取（无新测试，既有套件守护）

### Files
```
新 migrations/versions/20260801_000000_add_check_tables.py
改 src/models.py                                          # 追加 CheckSession / CheckItem
改 src/modules/warehouse/services/tube_service.py         # 提取 _insert_tube_with_compliance
```

### Interfaces

- `_insert_tube_with_compliance(conn, type_, outer_diameter_mm, wall_thickness_mm, length_m, steel_grade, location, operator_id, manufacturer, pipe_id=None, thread_type=None, api_5ct_cert=None, batch_no=None, heat_no=None, heat_treat_method=None, quality_mark="normal", quality_note=None, seamless_specs=None, screen_specs=None, compliance_pass=None) -> dict`
  - 接收**已打开连接**；调用方持有 begin_immediate/commit/rollback
  - 内部**无任何 rollback 调用**（失败仅 raise）；返回 `{"pipe_id", "transaction_id", "location", "compliance_pass"}`
- `stock_in(...)` 签名与返回**完全不变**（薄壳：connect → begin_immediate → 核心 → commit / except rollback+raise）
- ORM：`CheckSession`（location/status/created_by/confirmed_by/created_at/confirmed_at/remark）、`CheckItem`（check_id FK CASCADE + index/pipe_id/result/actual_location/surplus_data/created_at）

### Steps

**Step 1 — 迁移文件**（全文）：

```python
"""Add check_sessions + check_items for inventory check (盘点, D11).

Revision ID: 20260801_000000_add_check_tables
Revises: 8ce24f163996
Create Date: 2026-08-01
"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "20260801_000000_add_check_tables"
down_revision: Union[str, None] = "8ce24f163996"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "check_sessions",
        sa.Column("id", sa.Integer, primary_key=True, autoincrement=True),
        sa.Column("location", sa.String(32), nullable=False),
        sa.Column("status", sa.String(16), server_default=sa.text("'open'"), nullable=False),
        sa.Column("created_by", sa.Integer, nullable=False),
        sa.Column("confirmed_by", sa.Integer, nullable=True),
        sa.Column("created_at", sa.DateTime, nullable=False),
        sa.Column("confirmed_at", sa.DateTime, nullable=True),
        sa.Column("remark", sa.Text, nullable=True),
    )
    op.create_table(
        "check_items",
        sa.Column("id", sa.Integer, primary_key=True, autoincrement=True),
        sa.Column(
            "check_id",
            sa.Integer,
            sa.ForeignKey("check_sessions.id", ondelete="CASCADE"),
            nullable=False,
        ),
        sa.Column("pipe_id", sa.String, nullable=False),
        sa.Column("result", sa.String(16), nullable=False),
        sa.Column("actual_location", sa.String(32), nullable=True),
        sa.Column("surplus_data", sa.Text, nullable=True),
        sa.Column("created_at", sa.DateTime, nullable=False),
    )
    op.create_index("ix_check_items_check_id", "check_items", ["check_id"])


def downgrade() -> None:
    op.drop_index("ix_check_items_check_id", table_name="check_items")
    op.drop_table("check_items")
    op.drop_table("check_sessions")
```

**Step 2 — models.py 追加**（文件末尾；需 `from datetime import datetime, timezone` 已导入——检查并补齐）：

```python
# ---------------------------------------------------------------------------
# check_sessions / check_items  —  inventory check (盘点, D11)
# ---------------------------------------------------------------------------


class CheckSession(Base):
    __tablename__ = "check_sessions"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    location: Mapped[str] = mapped_column(String(32), nullable=False)
    status: Mapped[str] = mapped_column(String(16), nullable=False, default="open")  # open/confirmed/cancelled
    created_by: Mapped[int] = mapped_column(Integer, nullable=False)
    confirmed_by: Mapped[Optional[int]] = mapped_column(Integer, nullable=True)
    created_at: Mapped[datetime] = mapped_column(
        DateTime, default=lambda: datetime.now(timezone.utc), nullable=False
    )
    confirmed_at: Mapped[Optional[datetime]] = mapped_column(DateTime, nullable=True)
    remark: Mapped[Optional[str]] = mapped_column(Text, nullable=True)


class CheckItem(Base):
    __tablename__ = "check_items"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    check_id: Mapped[int] = mapped_column(
        Integer,
        ForeignKey("check_sessions.id", ondelete="CASCADE"),
        nullable=False,
        index=True,
    )
    pipe_id: Mapped[str] = mapped_column(String, nullable=False)
    result: Mapped[str] = mapped_column(String(16), nullable=False)  # ok/missing/moved/surplus
    actual_location: Mapped[Optional[str]] = mapped_column(String(32), nullable=True)
    surplus_data: Mapped[Optional[str]] = mapped_column(Text, nullable=True)
    created_at: Mapped[datetime] = mapped_column(
        DateTime, default=lambda: datetime.now(timezone.utc), nullable=False
    )
```

**Step 3 — tube_service.py 重构**（精确到块的机械重构，行为零变化）：

3a. 将现有 `stock_in` 的全部函数体（`async with eng.connect() as conn:` 之后的步骤 0-7，含 Transaction/审计写入与 `return {...}`）原样搬入新函数 `_insert_tube_with_compliance(conn, ...)`，**改动仅三处**：
- 函数签名首位加 `conn`（后续参数名与现有 `stock_in` 形参完全一致）；删除 `eng = engine or db.engine` 与 `async with eng.connect() as conn:` 包裹
- 删除函数体内**全部** `await conn.rollback()` 调用（共 4 处：compliance spec None、compliance 失败、uniqueness 冲突、location 不存在——每处保留其 `raise`）
- 删除末尾 `await conn.commit()`（保留 `return {...}`）

3b. `stock_in` 重写为薄壳（docstring 保留原文，追加一句"核心逻辑见 `_insert_tube_with_compliance`"）：

```python
    eng = engine or db.engine

    # Auto-generate pipe_id if not provided
    if pipe_id is None:
        pipe_id = _generate_pipe_id(type_, steel_grade, outer_diameter_mm, wall_thickness_mm)
    else:
        _validate_pipe_id_format(pipe_id, type_)

    async with eng.connect() as conn:
        await begin_immediate(conn)
        try:
            result = await _insert_tube_with_compliance(
                conn,
                type_,
                outer_diameter_mm,
                wall_thickness_mm,
                length_m,
                steel_grade,
                location,
                operator_id,
                manufacturer,
                pipe_id=pipe_id,
                thread_type=thread_type,
                api_5ct_cert=api_5ct_cert,
                batch_no=batch_no,
                heat_no=heat_no,
                heat_treat_method=heat_treat_method,
                quality_mark=quality_mark,
                quality_note=quality_note,
                seamless_specs=seamless_specs,
                screen_specs=screen_specs,
                compliance_pass=compliance_pass,
            )
            await conn.commit()
            return result
        except Exception:
            await conn.rollback()
            raise
```

**Step 4 — 验证（本任务无新测试，重构由既有套件守护）**：

```bash
DATABASE_URL=sqlite+aiosqlite:///data/test_migration_check.db .venv/bin/python -m alembic upgrade head   # 迁移链可应用
ls -la data/test_migration_check.db && rm -f data/test_migration_check.db
.venv/bin/python -m pytest tests/test_tubes_routes.py tests/test_stock_in_compliance.py tests/test_import_http.py tests/test_concurrent_stockout.py -v
# 预期：全绿（14 项入库 + 导入 + 并发测试），无任何行为差异
.venv/bin/python -m pytest -m "not slow"   # 全量回归：115 passed + 3 deselected
```

**预期输出**：迁移应用成功；重构前后两次全量回归结果完全一致（115 passed）。

---

