# 规格索引 — 从旧测试提取的行为契约

> 用途：重写期间唯一的行为来源。旧 Python 测试不可执行，但每个 test 的 docstring
> 写清了当年被验证过的行为；本目录把它们固化为编号条目，编码时逐条对照。
>
> 规则：
>
> 1. 一条规格 = 一个可写测试的断言。写不出测试的句子不算规格。
> 2. 每条必须标注来源（旧测试文件 + test 名），便于回查实现细节。
>
> 3. 实现完成即改状态为 ✅，并在 `docs/milestones/Mn.md` 引用编号。

## 编号

| 前缀 | 领域 | 来源文件 |
| --- | --- | --- |
| A | 认证与会话（口令、JWT、CSRF、限流、登出、轮换） | `test_security.py`、`test_login_lockout_audit.py` |
| B | 授权矩阵（角色 × 端点 × 方法） | `test_routes_protection.py` |
| C | 用户管理（创建/改密/吊销/角色变更审计） | `test_admin_audit.py`、`test_routes_protection.py` |
| D | 管子与库存流转 | `test_tubes_routes.py`、`test_batch_stockout.py`、`test_concurrent_stockout.py`、`test_used_count.py` |
| E | 合规（API 5CT） | `test_stock_in_compliance.py` |
| F | 审计链 | `test_audit_chain_20k.py`、`test_audit_route.py`、`test_panic_recovery.py` |
| G | 盘点 | `test_check.py` |
| H | 导入与上传上限 | `test_import_http.py`、`test_upload_limit.py` |
| I | 质量标注与库位 | `test_quality.py`、`test_tubes_routes.py` |

## 状态总览

| 文件 | 条目 | 已提取 | 已实现 |
| --- | --- | --- | --- |
| [`A-auth.md`](A-auth.md) | A1–A24 | 24 | 24（A13 已在 M6 补齐） |
| `B-authz.md` | B1–B10 | 待提取 | 0 |
| `C-users.md` | C1–C9 | 待提取 | 0 |
| [`D-inventory.md`](D-inventory.md) | D1–D16 | 16 | 16（M3） |
| [`E-compliance.md`](E-compliance.md) | E1–E8 | 8 | 8（M2/M3） |
| [`F-audit.md`](F-audit.md) | F1–F11 | 11 | 9（F10/F11 待 M5） |
| [`G-check.md`](G-check.md) | G1–G12 | 12 | 12（M4） |
| [`H-import.md`](H-import.md) | H1–H9 | 9 | 9（M0/M5） |
| `I-quality.md` | I1–I5 | 待提取 | 4（质量与库位已并入 D 域） |

优先级：A + B（M1）→ C（M1）→ D + E + F（M2/M3）→ G + H + I（M4/M5）。
