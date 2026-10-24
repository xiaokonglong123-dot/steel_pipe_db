# 审计缺口修复：用户管理 + 标准管理写操作审计

**日期**：2026-08-01
**状态**：待审阅
**设计权威**：`docs/2026-07-13-api5ct-inventory-design.md`（v0.4.0，2026-08-01）
**相关缺口**：设计文档 §17「用户管理/标准管理写操作审计」（2026-08-01 全量核对时记录）

---

## 1. 背景

设计文档 §4.7 将 `create_user` 列为审计 action 类型示例，即设计意图要求用户管理操作入审计链。但实际实现中 `_write_audit` 调用点仅存在于 warehouse 模块的管操作（`stock_in` / `stock_out` / `transfer`，位于 `tube_service.py`）：

- 用户管理 4 操作（`create_user` / `update_user` / `revoke_all_sessions` / `toggle_active`）不写审计
- standards 3 写操作（create / update / delete）不写审计，且 `delete_standard` 注释明确「参考数据，不审计」

本 spec 修复该缺口，使全部管理写操作进入审计哈希链。

## 2. 目标与非目标

### 目标

1. 用户管理 4 操作写入审计日志
2. standards 3 写操作写入审计日志
3. 全部满足强制约定 #13：**审计日志与业务操作在同一 `BEGIN IMMEDIATE` 事务内写入**，同事务提交/回滚
4. 操作者（actor）正确记录到 `audit_logs.user_id`
5. 失败路径零审计残留（回滚即消失）

### 非目标（明确排除）

- **登录锁定/解锁审计**：§17 另一独立缺口（涉及 auth 模块，无既有事务上下文），另行处理
- **启动播种 admin 审计**：`main.py` lifespan 系统引导创建，无操作者上下文，不审计
- **`audit_logs` 加 `target_type` / `target_id` 列**：现有 `action` + `detail` 已完整表达语义；加列需迁移 + 回填，YAGNI
- **`audit_logs` 查询/展示改造**：模板直接渲染 `action` 原文，无需映射

## 3. 设计

### 3.1 Action 命名与 detail 规范

沿用现有 `verb_noun` 风格（`stock_in` / `stock_out` / `transfer`）与设计文档 §4.7 示例（`create_user`）：

| 操作 | action | detail 内容 | 备注 |
|---|---|---|---|
| 创建用户 | `create_user` | `username=<name> role=<role>` | **绝不记录密码** |
| 编辑用户 | `update_user` | 仅变更字段，如 `role=admin`；密码变更记 `password_changed=true` | 不记录 hash 原文 |
| 撤销会话 | `revoke_sessions` | `user_id=<id> token_version+1` | |
| 启停账号 | `toggle_user` | `user_id=<id> active=<bool>` | |
| 新建标准 | `create_standard` | `grade=<g> od=<od> min_wt=<min> max_wt=<max>` | |
| 更新标准 | `update_standard` | 变更字段 | 与创建同字段集合 |
| 删除标准 | `delete_standard` | **硬删前快照**：`grade=<g> od=<od> min_wt=<min> max_wt=<max>` | 行删除后审计为唯一留存，数据保全 |

`AuditLog.user_id` = **操作者（actor）**，与管操作现有语义一致。

### 3.2 调用点与签名

7 个 service 函数均新增必填参数 `actor_id: int`（无默认值——避免「可跳过审计」路径，保持强制力）。唯一调用方是 users 路由 4 处与 standards 路由 3 处；测试无直调（已核查），`main.py` 播种不经 `create_user`。

### 3.3 user_service 改造（`src/modules/users/services/user_service.py`）

4 个函数已使用 `raw conn + begin_immediate`，仅需在业务写之后、`conn.commit()` 之前插入 `_write_audit(conn, action, detail, user_id=actor_id)`（`_write_audit` 从 `audit_service` 追加导入，跨模块 import 已有先例：`begin_immediate`）。

- `create_user`：INSERT 成功后、commit 前写审计
- `update_user`：仅在 `if values:` 分支内写审计（空变更 → 现状 rollback，无审计行）
- `revoke_all_sessions`：UPDATE 后写审计
- `toggle_active`：UPDATE 后写审计（`LastAdminError` 路径 rollback，无审计行）

### 3.4 standards_service 新建（`src/modules/warehouse/services/standards_service.py`）

与 user_service 完全同模式：`raw conn + begin_immediate` + `_write_audit`。

```
create_standard(steel_grade, outer_diameter_mm, min_wall_thickness_mm,
                max_wall_thickness_mm, actor_id, engine=None) -> None
update_standard(spec_id, steel_grade, outer_diameter_mm, min_wall_thickness_mm,
                max_wall_thickness_mm, actor_id, engine=None) -> None
delete_standard(spec_id, actor_id, engine=None) -> None
```

- 新异常 `StandardConflictError(Exception)`：重复钢级（`IntegrityError` 捕获后 rollback 再抛）→ 路由映射 409
- 不存在：`ValueError(f"standard spec {spec_id} not found")` → 路由映射 404
- `delete_standard`：DELETE 前先 SELECT 快照，审计 detail 写入被删值，再 DELETE + `_write_audit` + commit

### 3.5 standards 路由瘦身（`src/modules/warehouse/routers/standards.py`）

3 个 write handler 改为调用 service，保留现有 UI/错误行为：

- `POST /standards`：catch `StandardConflictError` → 渲染 409（现有「该钢级已存在」表单重渲染保持）；成功 → 303
- `POST /standards/{spec_id}`：catch `StandardConflictError` → `JSONResponse 409`（现有行为）；`ValueError` → 404
- `POST /standards/{spec_id}/delete`：`ValueError` → 404；移除「not audited」注释
- 只读路由（list / edit form）不动

### 3.6 事务与回滚语义

所有 7 个操作：`BEGIN IMMEDIATE` → 业务写 → `_write_audit` → `commit`。任一失败 → `rollback`，审计行一并消失。哈希链在并发下无碎片（`BEGIN IMMEDIATE` 串行化保证 `prev_hash` 为最新已提交链尾，与现有管操作一致）。

## 4. 测试计划（TDD，新文件 `tests/test_admin_audit.py`）

| # | 测试 | 断言 |
|---|---|---|
| 1 | `test_create_user_writes_audit` | 审计行存在：action=`create_user`、detail 含 username/role、user_id=actor |
| 2 | `test_update_user_writes_audit` | 变更 role → detail 含 `role=`；user_id=actor |
| 3 | `test_update_user_password_hides_hash` | detail 含 `password_changed=true` 且**不含** argon2 `$argon2` 前缀 |
| 4 | `test_update_user_no_changes_no_audit` | 空变更 → 无新审计行 |
| 5 | `test_revoke_sessions_writes_audit` | action=`revoke_sessions`、detail 含 `token_version+1` |
| 6 | `test_toggle_user_writes_audit` | action=`toggle_user`、detail 含 `active=False` |
| 7 | `test_duplicate_username_no_audit` | `UserConflictError` → 无审计行 |
| 8 | `test_toggle_last_admin_rejected_no_audit` | `LastAdminError` → 无审计行 |
| 9 | `test_create_standard_writes_audit` | action=`create_standard`、detail 含 grade/od/wt |
| 10 | `test_update_standard_writes_audit` | action=`update_standard`、detail 含变更字段 |
| 11 | `test_delete_standard_writes_audit_with_snapshot` | action=`delete_standard`、detail 含被删 grade/od/wt 快照 |
| 12 | `test_duplicate_grade_no_audit` | `StandardConflictError` → 无审计行 |
| 13 | `test_chain_verifies_after_mixed_ops` | 混合操作后 `POST /audit/verify` → ok=true（链连续性） |

测试文件沿用现有模式：独立 test DB、`sys.path.insert`、env 预置、`app.dependency_overrides`（`get_current_user` / `require_csrf` stub）。

## 5. 文档同步（实现完成后）

- **设计文档 → v0.5.0**：
  - §4.7：action 类型清单补全（7 种管理操作）
  - §17：移除「用户管理/标准管理写操作审计」缺口行；移除/改写「standards 硬删不审计」取舍行（现改为审计并快照）
  - §13：测试矩阵补 `test_admin_audit.py` 行、基线数更新
  - §0.4：现「本次修订（v0.3.0 → v0.4.0）」标题改为「历史修订（v0.3.0 → v0.4.0，2026-08-01 归档）」；新增「§0.4 本次修订（v0.4.0 → v0.5.0）」；状态总览保持 §0.5；grep 修正文档内「§0.4」指代
- **AGENTS.md**：NOTES 补审计覆盖范围说明
- **应用版本**：0.3.0 → 0.4.0（新功能版本，`pyproject.toml` + `main.py`）

## 6. 文件清单

| 文件 | 动作 |
|---|---|
| `src/modules/users/services/user_service.py` | 改：4 函数 + `actor_id` + `_write_audit` |
| `src/modules/users/routers/users.py` | 改：4 处调用传 `actor_id=user.id` |
| `src/modules/warehouse/services/standards_service.py` | **新建**：3 函数 + `StandardConflictError` |
| `src/modules/warehouse/routers/standards.py` | 改：3 write handler 瘦身 |
| `tests/test_admin_audit.py` | **新建**：13 项测试 |
| `docs/2026-07-13-api5ct-inventory-design.md` | 改：v0.5.0 同步 |
| `AGENTS.md` | 改：NOTES 审计覆盖 |

## 7. 风险与注意事项

- **跨模块依赖**：users → warehouse 的 `audit_service` import 已有先例（`begin_immediate`），无循环 import 风险（`audit_service` 不 import users/warehouse 业务模块）
- **`actor_id` 必填**：破坏性签名变更，但调用方仅 7 处且全部可传值；无其他直调（已核查）
- **standards 事务语义变更**：AsyncSession → raw conn，错误行为（409/404）由路由层保持，回归由测试 9-12 兜底
