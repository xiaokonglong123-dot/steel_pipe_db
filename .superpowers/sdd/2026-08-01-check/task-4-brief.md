## Task 4: 文档同步（design v0.7.0 + AGENTS.md v0.7.0 + 版本 0.6.0）

### Files
```
改 docs/2026-07-13-api5ct-inventory-design.md
改 AGENTS.md
改 pyproject.toml
改 src/main.py
改 docs/superpowers/specs/2026-08-01-check-design.md     # 状态 → 已实现
```

### Steps

**Step 1 — 设计文档 v0.6.0 → v0.7.0**：

1a. 头部版本/日期行：`v0.6.0` → `v0.7.0`（日期 2026-08-01）
1b. `### 0.6 本次修订（v0.5.0 → v0.6.0，2026-08-01）` 标题改 `### 0.6 历史修订（v0.5.0 → v0.6.0，2026-08-01 归档）`
1c. 在其后新增 `### 0.7 本次修订（v0.6.0 → v0.7.0，2026-08-01）` 修订记录表，D11 行：盘点 /tubes/check 实现（check_sessions/check_items 两表、missing 状态、单事务原子确认、盘盈走合规、recover 找回、8 端点、3 页面）
1d. 原 `### 0.7 实现状态总览（2026-08-01）` → `### 0.8 实现状态总览（2026-08-01）`；总览表内新增：盘点流程 ✓、check_sessions/check_items ✓；`grep -n "0\.7\|0\.6"` 修正当前文本指代（**归档行保留原编号**——历史修订行里的章节号是历史事实，勿改）
1e. §5.3 盘点流程：标题下加"（已实现 2026-08-01，D11）"，并把流程草图的决策补全为一句：盘点单 DB 持久化（check_sessions/check_items）；盘亏 → status=missing；确认在单个 BEGIN IMMEDIATE 内原子应用（任一项失败整体回滚、session 保持 open）；盘盈走 compliance；找回 recover；仅 confirm/recover 写审计
1f. §3.1 schema DDL 补两表 DDL（与迁移一致：字段/类型/FK CASCADE/ix_check_items_check_id）；测试矩阵：文件数 13 → 14（加 test_check.py 行：D11 盘点，22 项）、基线 **112/109+3 → 140/137+3**（共 140 项：137 通过 + 3 slow）
1g. §5.1 模板/页面：13 → 16（加 warehouse/check_list.html、check_detail.html、check_report.html）
1h. §4.4 端点表：补 10 行（GET/POST /tubes/check、GET /tubes/check/{id}、POST items、POST surplus、POST surplus/{id}/delete、GET report、POST confirm、POST cancel、POST /tubes/recover/{pipe_id}；角色：核对/标记/盘盈/删除/取消/报告 operator，确认 admin）
1i. §17 缺口表：移除「盘点 /tubes/check（含盘盈走合规）」行
1j. 附录 A：文档版本 v0.6.0 → v0.7.0、基线 115 → 137
1k. 自检：`grep -n "v0\.6\.0\|115\|13 文件\|13 页"` 无残留（归档行除外）；`grep -n "盘点"` 仅修订记录与 5.3/端点表/17 已移除处命中

**Step 2 — AGENTS.md v0.6.0 → v0.7.0**：
- 头部版本/日期；结构树：`warehouse/services/` 补 `check_service.py`；templates 补 3 个 check_*.html
- NOTES 补：盘点语义（check_sessions/check_items 持久化；盘亏 status='missing'（新状态值，非 out_stock）；确认单 BEGIN IMMEDIATE 原子应用（失败整体回滚、session 保持 open、零审计）；盘盈走 compliance；recover 找回；仅 confirm/recover 写审计）；测试基线 115 → **137 passed + 3 deselected**

**Step 3 — 版本**：
- `pyproject.toml` `version = "0.5.0"` → `"0.6.0"`
- `src/main.py` `FastAPI(version="0.5.0", ...)` → `"0.6.0"`（**无 __version__ 变量**，只改构造参数）

**Step 4 — spec 状态**：`2026-08-01-check-design.md` 头部 `**状态**: 已确认（用户审批）` → `**状态**: 已实现（2026-08-01，D11）`

**Step 5 — 验证**：

```bash
grep -n "0\.5\.0\|v0\.6\.0" docs/2026-07-13-api5ct-inventory-design.md AGENTS.md pyproject.toml src/main.py | grep -v "历史修订\|归档\|0\.7" || true   # 无残留（归档行除外）
.venv/bin/python -m pytest -m "not slow"   # 136 passed + 3 deselected（文档改动不影响）
```

**预期输出**：grep 无残留；全量回归 136 passed。

---

## 收尾（全部任务完成后由控制器执行）

1. 最终全分支审查（最强大模型，以计划为唯一真相源，允许自行跑全量回归）
2. 更新 ledger + 项目记忆（基线 136、文档 v0.7.0、版本 0.6.0、盘点语义）
3. 向用户汇报（含 ⚠️ 裁决：missing 状态对现有查询的语义影响（新状态值不影响现有 status='in_stock' 守卫与 COUNT 语义）、check 确认对 stock_in 审计行的影响（盘盈核心在确认事务内写 stock_in 行，属预期留痕）、确认并发语义（TOCTOU 守卫回滚））
