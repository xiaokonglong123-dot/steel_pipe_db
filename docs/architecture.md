# 架构与业务规则

本文说明当前 Rust 实现的边界和不变量。可执行行为见 [`spec.md`](spec.md)，接口字段以 `openapi.json` 为准，数据库结构以 `crates/infra/migrations/` 为准。

## 架构

```text
React SPA ──JSON / httpOnly cookie──> axum API
                                      │
                           service 用例与权限校验
                                      │
                    domain 状态机 / compliance / audit
                                      │
                         infra 事务、仓储、迁移
                                      │
                              SQLite（WAL）
```

| crate | 职责与限制 |
| --- | --- |
| `domain` | 角色、枚举、状态机；无 IO、Web 或运行时依赖 |
| `compliance` | API 5CT 五项纯函数校验 |
| `audit` | 哈希计算、紧凑 detail、分批验证 |
| `auth` | argon2id、HS256；密钥显式注入 |
| `infra` | SQLite 连接、迁移、`Uow` 写事务、仓储 |
| `service` | 用例编排；不依赖 axum |
| `api` | HTTP、DTO、错误映射、OpenAPI；不直接依赖 sqlx/infra |
| `server` | 配置、静态资源、CLI、进程装配 |

`scripts/ci/layering.sh` 用直接依赖图验证边界。前端构建产物由 `rust-embed` 内嵌，部署只需二进制、数据库及运维脚本。

## 业务规则

### 管子与库位

- 类型为 `seamless_pipe` / `screen_pipe`，状态为 `in_stock` / `in_transfer` / `out_stock` / `missing`。
- 质量标记独立于库存状态：`normal` / `problematic` / `scrapped`。
- 问题管出库、调拨需管理员；报废管不得出库或调拨。
- 管子采用软删除，默认列表隐藏已删除记录；在库管软删时释放计数。
- 库位通过启停用管理；停用后不能入库或作为调拨目标。
- `used_count` 是在库、未删除管子的派生计数，必须与实际状态一致，可由 `reconcile` 修复。

### 合规

入库与盘盈使用同一套写路径和校验：

1. 钢级精确匹配；
2. 壁厚位于闭区间，允许只给单侧界限；
3. 有标准外径时，偏差不超过 ±1%；
4. 长度为正；
5. 筛管必须提供参数，槽宽和过滤精度为正。

无匹配标准规格不得跳过校验。不合规入库只能由管理员 `force`，并额外记录 `forced_intake` 审计。盘盈不开放强制入库。

### 盘点

- 盘点单状态：`open` → `confirmed` / `cancelled`。
- 明细为 `ok` / `missing` / `moved` / `surplus`；同一单同一管只保留一条结论。
- 登记明细时校验范围与目标库位；关闭后不得新增、覆盖或删除明细。
- 确认时重新读取明细、校验盘盈合规和移库目标，在单事务内应用差异、写单据与审计。
- 任一项失败整体回滚，单据保持 `open`。
- 找回只允许 `missing` → `in_stock`，同时恢复库位计数并写审计。

## 数据模型

| 表 | 作用 |
| --- | --- |
| `tubes`、`seamless_pipe_specs`、`screen_pipe_specs` | 管子主记录及类型扩展 |
| `locations` | 库位、启停用与派生计数 |
| `standard_specs` | 钢级合规标准，grade 唯一 |
| `transactions`、`transaction_items` | 出入库、调拨、盘点单据及历史 |
| `check_sessions`、`check_items` | 盘点单、暂存结论及盘盈数据 |
| `audit_logs`、`audit_chain_tip` | 审计行与单行链尾 |
| `users`、`login_attempts` | 身份、会话版本与持久化登录限流 |

时间由应用写入统一 RFC3339 微秒格式。历史单据的 `pipe_id` 不引用管子外键，避免管子生命周期影响历史留档。迁移只能追加，不修改已部署迁移的历史含义。

## 并发与审计

- 所有状态变更通过 `Uow` 的 `BEGIN IMMEDIATE`，业务行和审计同事务提交。
- 状态读取与条件检查必须在同一写事务；不能“先在池上读，再开事务写”。
- 状态推进使用条件 SQL 并检查影响行数：状态、原库位、软删除、质量或 `open` 状态不满足时返回冲突，不能用旧快照调整计数。
- 计数增减与目标库位启用条件放在同一 SQL 中。
- 审计链为 `sha256(prev_hash ‖ action ‖ detail ‖ timestamp)`，创世值为 `sha256("AYANAMI_REI_GENESIS")`。
- 校验使用单连接读事务快照，按 id 分批，跨批传递 tip，并与同一快照中的链尾比较。

## 认证与接口

- 口令使用 argon2id，access token 15 分钟，refresh token 30 天。
- access/refresh 通过 httpOnly、`SameSite=Lax` cookie 下发；CSRF 使用非 httpOnly double-submit cookie 与 `X-CSRF-Token`。
- refresh jti 与用户当前记录比对，轮换后旧值失效；改密、停用和会话版本用于吊销访问。
- 登录失败按 IP 和用户名限流并持久化；默认不信任转发头。
- 角色为 `admin` / `operator` / `viewer`：查看者只读，操作员处理库存，管理员处理目录、用户、审计校验及盘点确认。
- 错误采用 RFC 9457 Problem Details；OpenAPI 是前端类型生成来源。
