### Task 4: 文档同步 + 版本 bump

**Files:**
- Modify: `docs/2026-07-13-api5ct-inventory-design.md`
- Modify: `AGENTS.md`
- Modify: `pyproject.toml`
- Modify: `src/main.py`
- Modify: `docs/superpowers/specs/2026-08-01-audit-gap-design.md`（状态行更新为已实现）

**Interfaces:**
- 无代码接口。文档数字与 v0.5.0 修订记录必须与实际一致。

- [ ] **Step 1: 设计文档升 v0.5.0**

修改 `docs/2026-07-13-api5ct-inventory-design.md`：

(a) 头部版本行：
`> 版本：v0.4.0（v0.3.0 同步修订版：代码全量核对 + 漂移修正 + 测试基线更新）` → `> 版本：v0.5.0（v0.4.0 功能版：用户/标准管理写操作审计落地）`；日期行 → `> 日期：2026-08-01`。

(b) §0.4（现「本次修订（v0.3.0 → v0.4.0）」）标题改为：`### 0.4 历史修订（v0.3.0 → v0.4.0，2026-08-01 归档）`。

(c) 在 §0.4 之后、§0.5（状态总览）之前插入新节：

```markdown
### 0.5 本次修订（v0.4.0 → v0.5.0，2026-08-01）

用户管理与标准管理写操作审计落地（D9，对应 spec `docs/superpowers/specs/2026-08-01-audit-gap-design.md`）：

| # | 类型 | 位置 | 变更说明 |
|---|---|---|---|
| 1 | 功能 | §4.7 | 审计 action 清单补全：create_user / update_user / revoke_sessions / toggle_user / create_standard / update_standard / delete_standard；AuditLog.user_id = 操作者 |
| 2 | 功能 | §17 | 移除「用户管理/标准管理写操作审计」缺口行；standards 硬删改为**审计 + 删前快照**（不再「不审计」） |
| 3 | 结构 | §6 | 新增 `warehouse/services/standards_service.py` |
| 4 | 测试 | §13 | 新增 test_admin_audit.py（13 项）；基线 96 → 109 通过 + 3 slow |
| 5 | 版本 | §0.1 | 应用版本号 0.3.0 → 0.4.0 |
```

（§0.5 状态总览重编号为 §0.6，`grep -n "§0.5"` 修正指代；`grep -n "§0.4"` 确认其余指代指向正确章节。）

(d) §4.7 末尾的「实现现状（v0.4.0 记录）」警告注记替换为：

```markdown
> ✅ **实现现状（v0.5.0）**：审计覆盖 warehouse 管操作 + 用户管理 4 操作 + standards 3 写操作（v0.5.0 D9 落地）。登录锁定/解锁审计仍为 §17 缺口。
```

(e) §13.1 测试矩阵（现「共 99 项：96 通过 + 3 项 slow」）更新为「共 112 项：109 通过 + 3 项 slow」，并在表格中追加一行：

```markdown
| test_admin_audit.py | 管理写操作审计：用户 4 操作 + standards 3 操作 → 审计行；失败路径零审计；密码哈希不入 detail；混合操作后链验证 ok | 13 |
```

（若矩阵行无数量列，则仅追加覆盖描述；基线行数字 96 → 109 必改。）

(f) §17 已知取舍中「standards 硬删（参考数据，不审计）」行改为「standards 硬删（审计 + 删前快照，v0.5.0 起）」；删除功能缺口表中的「用户管理/标准管理写操作审计」行。

(g) 附录 A 文档引用行更新为 v0.5.0。

- [ ] **Step 2: AGENTS.md 同步**

修改 `AGENTS.md`：
- `**Generated:** 2026-08-01`（保持）；`**Version:** v0.4.0` → `v0.5.0`
- 结构树中 `warehouse/services/` 行下补 `standards_service.py  # API 5CT 标准规格 CRUD + 审计（v0.5.0）`
- NOTES 中「Test baseline」行更新为 `(2026-08-01): 109 passed + 3 deselected`
- NOTES 追加一行：`Audit covers warehouse tube ops + user management (create_user/update_user/revoke_sessions/toggle_user) + standards writes (create/update/delete with pre-delete snapshot); AuditLog.user_id is the actor (v0.5.0, D9).`

- [ ] **Step 3: 版本 bump 0.4.0**

- `pyproject.toml` line 3：`version = "0.3.0"` → `version = "0.4.0"`
- `src/main.py` line 172：`version="0.3.0"` → `version="0.4.0"`

- [ ] **Step 4: 回归验证**

Run: `.venv/bin/python -m pytest -m "not slow" -q`
Expected: 109 passed, 3 deselected。
再跑一次 grep 自检：
`grep -n "96 passed\|95 passed\|56 项\|53 通过" docs/2026-07-13-api5ct-inventory-design.md AGENTS.md` → 预期无匹配（旧基线数字全部清除）；`grep -rn "0.2.2" pyproject.toml src/main.py` → 预期无匹配。

- [ ] **Step 5: 更新 spec 状态行**

`docs/superpowers/specs/2026-08-01-audit-gap-design.md` 头部 `**状态**：待审阅` → `**状态**：已实现（2026-08-01，D9 落地，见设计文档 v0.5.0）`。
