# 库存盘点（D11）设计 — spec

**日期**: 2026-08-01
**状态**: 已实现（2026-08-01，D11）
**设计权威**: docs/2026-07-13-api5ct-inventory-design.md §5.3（流程草图，本 spec 决策完备化）

---

## 1. 目标

实现库存盘点 `/tubes/check`：选择库位 → 逐根核对（一致/缺失/位置异常/盘盈）→ 生成报告 → 管理员确认后原子应用全部差异（事务 + 审计）。设计文档 §5.3 流程草图 → 本 spec 补齐所有决策：

- 盘点单 **DB 持久化**（check_sessions + check_items，可保存/恢复/查历史）
- 盘亏用**新状态 `missing`**（非复用 out_stock）
- 确认**原子性**：单 `BEGIN IMMEDIATE` 应用全部差异，任一项失败整体回滚
- 盘盈走 **compliance 门**（同入库，缺口注明"盘盈走合规"）
- 找回恢复：missing → in_stock + 审计

## 2. 非目标

- 批量筛选出库、质量标注页、库位管理页、批次查询、Excel 导入、forced_intake、临近库位提示——其余 §17 缺口，另案
- 盘点单的定时/计划盘点（无频率概念）
- 盘点期间锁定库存（盘点是快照核对，不阻止并发业务；并发冲突由确认时 TOCTOU 守卫兜底）

## 3. 数据模型（迁移 4：纯新表）

```
check_sessions:
  id            INTEGER PK autoincrement
  location      TEXT NOT NULL          # 盘点库位
  status        TEXT NOT NULL DEFAULT 'open'   # open / confirmed / cancelled
  created_by    INTEGER NOT NULL       # 操作者（非 FK，与 tubes.operator_id 同惯例）
  confirmed_by  INTEGER NULL
  created_at    DATETIME NOT NULL
  confirmed_at  DATETIME NULL
  remark        TEXT NULL

check_items:
  id              INTEGER PK autoincrement
  check_id        INTEGER NOT NULL FK→check_sessions.id (ondelete CASCADE)
  pipe_id         TEXT NOT NULL        # 盘盈项在添加时即生成预览 pipe_id
  result          TEXT NOT NULL        # ok / missing / moved / surplus
  actual_location TEXT NULL            # moved 时实际库位（必填且 ≠ 盘点库位）
  surplus_data    TEXT NULL            # surplus 完整管信息 JSON（确认时解析走 stock_in 核心）
  created_at      DATETIME NOT NULL

  CHECK 约束: result IN ('ok','missing','moved','surplus')
```

- `Transaction.type_='check'` 已存在于 schema（String(16)），无需改动
- 迁移必须遵循既有命名/风格（时间戳前缀；新表用 `op.create_table`，无需 batch_alter_table）

## 4. 核心实现

### 4.1 stock_in 核心提取（受现有测试守护的重构）

`tube_service.stock_in`（步骤 0-7：compliance 门/唯一性/库位校验/INSERT tube/扩展表/used_count+1/Transaction+Item/审计）提取为：

```python
async def _insert_tube_with_compliance(
    conn, type_: str, outer_diameter_mm: float, wall_thickness_mm: float,
    length_m: float, steel_grade: str, location: str, operator_id: int,
    manufacturer: str, pipe_id: str | None = None, thread_type: str | None = None,
    api_5ct_cert: str | None = None, batch_no: str | None = None,
    heat_no: str | None = None, heat_treat_method: str | None = None,
    quality_mark: str = "normal", quality_note: str | None = None,
    seamless_specs: dict | None = None, screen_specs: dict | None = None,
    compliance_pass: bool | None = None,
) -> dict:
```

- 接收**已打开连接**（调用方持有 begin_immediate/commit/rollback）
- 内部所有 `await conn.rollback()` 调用**移除**（仅 raise，由调用方回滚）
- `stock_in` 变薄壳：`connect → begin_immediate → try: _insert_tube_with_compliance(conn, ...) → commit → return` / `except: rollback; raise`——签名与返回完全不变
- 现有 14 项 stock-in 测试 + CSV import 路径（compliance_pass 显式传参）守护

### 4.2 check_service.py（新，raw conn + begin_immediate 模式，同 user_service 风格）

```
create_check(location, operator_id, remark=None) -> CheckSession
mark_item(check_id, pipe_id, result, actual_location=None, actor_id)   # upsert；非 open → CheckConflictError(409)
add_surplus(check_id, surplus_data, actor_id) -> CheckItem             # 生成 pipe_id 预览存 surplus_data JSON + pipe_id
delete_surplus_item(check_id, item_id, actor_id)                       # 仅 surplus 项可删；非 open → 409
get_check(check_id), list_checks()
confirm_check(check_id, actor_id)                                      # 原子应用全部差异（见 4.3）
cancel_check(check_id, actor_id)                                       # open → cancelled；已确认 → 409
recover_missing(pipe_id, actor_id)                                     # missing → in_stock
```

新异常：`CheckConflictError`（409，表单重渲染语义同 ConflictError）

### 4.3 confirm_check 原子应用（单 BEGIN IMMEDIATE）

```
begin_immediate
1. 锁读 session（status='open' 否则 CheckConflictError）；加载全部 items
2. 逐项（任一失败 → raise → 整体回滚，session 保持 open，零审计零部分应用）：
   - ok        → 跳过
   - missing   → UPDATE tubes SET status='missing', updated_at=now
                  WHERE pipe_id=:p AND location=:loc AND status='in_stock'
                    AND deleted_at IS NULL AND quality_mark != 'scrapped'
                  rows_affected==1 否则 CheckConflictError(409)
                UPDATE locations SET used_count=used_count-1
                  WHERE code=:loc AND used_count>0
   - moved     → 校验 actual_location 存在且 active（否则 409）
                  UPDATE tubes SET location=:actual, updated_at=now
                  WHERE pipe_id=:p AND location=:loc AND status='in_stock'
                    AND deleted_at IS NULL AND quality_mark != 'scrapped'
                  rows_affected==1 否则 409
                UPDATE locations SET used_count=used_count-1 WHERE code=:loc AND used_count>0
                UPDATE locations SET used_count=used_count+1 WHERE code=:actual
   - surplus   → _insert_tube_with_compliance(conn, ...parse(surplus_data)...,
                  location=session.location, compliance_pass=None)   # 盘盈走合规
3. INSERT Transaction(type='check', operator_id=actor_id, remark=f"check {check_id}") + 每差异项一条 TransactionItem
4. _write_audit(conn, "check", f"check_id={id} location={loc} missing={n} surplus={n} moved={n}", actor_id)
5. UPDATE check_sessions SET status='confirmed', confirmed_by=:actor, confirmed_at=now
6. commit
```

- 确认者即 admin（路由层 require_admin）——moved 项无 problematic 限制（transfer 的 problematic 规则因操作者是 admin 而天然满足）
- 无差异（全部 ok 或空 items）→ 仅写 transaction+审计？**否**：无差异时跳过 2，仍写审计 `missing=0 surplus=0 moved=0` + session confirmed（报告留痕）
- 盘盈 surplus 的 compliance 失败在确认时抛出 → 整体回滚

### 4.4 recover_missing

```
begin_immediate
UPDATE tubes SET status='in_stock', updated_at=now
  WHERE pipe_id=:p AND status='missing' AND deleted_at IS NULL
rows_affected==1 否则 CheckConflictError(409)
UPDATE locations SET used_count=used_count+1 WHERE code=(tube.location)   # 先查 tube 取 location
INSERT Transaction(type='check', remark='recover') + item
_write_audit(conn, "recover", f"pipe_id={p} location={loc}", actor_id)
commit
```

## 5. 路由与权限（tubes.py 追加，全部 require_csrf）

| 路由 | 方法 | 权限 | 行为 |
|---|---|---|---|
| `/tubes/check` | GET | require_operator | 盘点列表 + 新建表单（库位下拉） |
| `/tubes/check` | POST | require_operator | create_check → 303 `/tubes/check/{id}` |
| `/tubes/check/{id}` | GET | require_operator | 核对页：库位在库管列表（status='in_stock'）+ 每根核对按钮 + 盘盈表单（同入库字段）+ 当前项状态 |
| `/tubes/check/{id}/items` | POST | require_operator | mark_item；冲突/非法 → 表单重渲染 409/422 |
| `/tubes/check/{id}/surplus` | POST | require_operator | add_surplus（录入时仅格式校验，compliance 留到确认） |
| `/tubes/check/{id}/surplus/{item_id}/delete` | POST | require_operator | delete_surplus_item |
| `/tubes/check/{id}/report` | GET | require_operator | 差异汇总报告 + 确认/取消按钮（确认按钮仅 admin 可见） |
| `/tubes/check/{id}/confirm` | POST | **require_admin** | confirm_check；ComplianceError→422、CheckConflictError→409，失败 → 报告页重渲染显示错误 |
| `/tubes/check/{id}/cancel` | POST | require_operator | cancel_check |
| `/tubes/recover/{pipe_id}` | POST | require_operator | recover_missing；非 missing → 409 重渲染 |

## 6. 模板（3 新，13 → 16 页面）

`warehouse/check_list.html`、`warehouse/check_detail.html`（核对+盘盈）、`warehouse/check_report.html`（报告+确认+错误显示）；沿用 base.html design token（--accent indigo、.surface、.btn-accent），无 emoji

## 7. 测试计划（新 `tests/test_check.py`，~20 项，基线 115 → ~135）

复用 test_admin_audit/test_tubes_routes 基建（独立 DB、_seed 用户+库位+标准规格+在库管、_allow_csrf/_stub_user、fx 重置；确认需 admin stub）

1. viewer GET /tubes/check → 403
2. viewer POST /tubes/check → 403
3. operator 创建 → 303 + 单存在 open
4. 标记 ok → item 存在
5. 标记 missing → item result=missing
6. 标记 moved 无 actual_location → 422（表单缺字段）
7. 标记 moved actual_location=原库位 → 409（CheckConflictError）
8. 标记 moved 合法 → item ok
9. upsert：同管重新标记 → 单行更新
10. 添加盘盈 → item 存在 + pipe_id 预览格式（`{SP|SCP}-{grade}-{od:.1f}x{wt:.2f}-uuid8`）+ surplus_data
11. 删除盘盈项 → 行消失；删非 surplus 项 → 409
12. 非 open 单标记/添加 → 409
13. operator 确认 → 403
14. admin 确认无差异单 → 303 + confirmed + 审计 check(missing=0 surplus=0 moved=0)
15. admin 确认 mixed（missing+moved+surplus 合规）→ 全部生效：missing 管 status=missing + used_count-1；moved 管 location=actual + used_count 迁移；surplus 管 in_stock + used_count+1 + compliance_pass=True；transaction(type=check)+items；审计 action=check + 盘盈 stock_in 行；session confirmed
16. 原子性：确认含不合格盘盈（无标准规格钢级）→ **422**（ComplianceError）+ session 仍 open + **零变化**（missing 管未变 missing、零审计行）
17. TOCTOU：确认前标记管已被出库 → 整体回滚 + session open
18. 取消 open → cancelled；确认已取消单 → 409
19. recover：missing 管 → in_stock + used_count+1 + 审计 recover + transaction；非 missing → 409
20. 混合全流程后 POST /audit/verify → 哈希链完整

## 8. 文档同步（实现完成后）

- 设计文档 **v0.7.0**：§5.3 标记已实现（流程补全决策：missing 状态/原子确认/盘盈合规/找回）；§3.1 schema 补 check_sessions/check_items；§5.1 页面 13→16；§3.1 测试矩阵 13→14 文件、118→~138 项；§4.4 端点表补 10 路由；§17 移除「盘点 /tubes/check（含盘盈走合规）」缺口行；新修订记录节 + 状态总览重编号（沿用 v0.6.0 先例：本次修订→历史修订、状态总览 → 下一节号）
- AGENTS.md **v0.7.0**：结构树补 check_service.py + 3 模板；NOTES 补盘点语义（missing 状态/原子确认）；测试基线 115→~135
- 应用版本 **0.6.0**（pyproject + main.py `FastAPI(version=...)`）
- spec 状态 → 已实现

## 9. 文件清单

```
新 migrations/versions/<ts>_check_sessions.py     # 两表 create_table
改 src/models.py                                  # CheckSession/CheckItem ORM
改 src/modules/warehouse/services/tube_service.py # 提取 _insert_tube_with_compliance
新 src/modules/warehouse/services/check_service.py
改 src/modules/warehouse/routers/tubes.py         # 10 新路由
新 src/templates/warehouse/check_list.html
新 src/templates/warehouse/check_detail.html
新 src/templates/warehouse/check_report.html
新 tests/test_check.py                            # ~20 项
文档同步（实现后）：design v0.7.0 / AGENTS.md v0.7.0 / 版本 0.6.0
```

## 10. 风险与边界

- **stock_in 重构**：核心提取受现有 14 项入库测试 + import 测试守护；行为零变化（仅移走 rollback 到调用方）
- **确认期间并发**：管在确认前被出库/调拨 → rows_affected=0 → 整体回滚，操作者解决后重确认（正确建模：盘点是快照核对）
- **审计防刷**：仅确认（每单 1 行 check + 盘盈的 stock_in 行）与 recover（每管 1 行）；创建/取消/标记不审计（盘点单表本身留痕，避免噪音）
- **内存**：check 无内存状态（全 DB 持久化），重启安全
