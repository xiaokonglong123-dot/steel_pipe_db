# AGENTS.md — SteelPipe / Ayanami_Rei

面向局域网的无缝钢管与筛管（API 5CT）单根级库存系统。单二进制 + SQLite，无 Python / Node 运行时依赖。

- 设计权威：`docs/2026-07-13-api5ct-inventory-design.md`
- 重写方案：`docs/2026-11-17-rewrite-reference.md`
- 行为规格：`docs/spec/`（A–H 域，编号可追溯）

## 架构

```text
crates/
  domain/        角色与业务枚举、管子状态机（无 IO）
  compliance/    API 5CT 五项校验（纯函数）
  audit/         哈希链计算与分批校验（纯函数）
  auth/          argon2id + HS256（Keys 显式持有）
  infra/         SQLite 连接池、PRAGMA、迁移、Uow（写事务 + 审计）、仓储
  service/       用例：认证/用户/库存/目录/盘点/导入/运维
  api/           axum 路由、DTO、RFC 9457 错误、OpenAPI（不依赖 sqlx）
  server/        单二进制装配、CLI、rust-embed 静态资源
web/             Vite 8 + React 19 + TypeScript SPA（构建期内嵌）
scripts/ci/      分层断言
deploy/          systemd 单元
```

分层由 CI 断言：`crates/api` 直接依赖中出现 `sqlx` 或 `steelpipe-infra` 即失败。

## 命令

```bash
cargo build --release --workspace --bins     # target/release/steelpipe
(cd web && npm ci && npm run build)          # web/dist，需重新 cargo build 才会内嵌
cargo test --workspace                       # 118 项
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/ci/layering.sh
(cd web && npm run typecheck && npm run build)
```

开发时前端可单独跑 `cd web && npm run dev`（5173，`/api` 代理到 8080）。

## 必须遵守

1. **单写者**：只启动一个 `steelpipe serve` 实例。SQLite 文件级特性 + 审计链串行化前提。
2. **写事务**：所有状态变更走 `BEGIN IMMEDIATE`；审计行与业务操作同事务写入。
3. **TOCTOU 守卫**：`WHERE status='in_stock' AND quality_mark != 'scrapped'` + 校验 `rows_affected == 1`。
4. **审计链**：`sha256(prev‖action‖detail‖ts)`；genesis = `sha256("AYANAMI_REI_GENESIS")`；校验分批（1000）。
5. **软删**：管子与库位不做物理删除；库位 FK `ON DELETE RESTRICT`。
6. **合规门**：所有入库（含盘盈）必须过 API 5CT 校验；不合规只能管理员 `force` 并留审计。
7. **盘点**：确认在单事务内原子应用全部差异，任一项失败整体回滚且盘点单保持 open。
8. **认证**：JWT 仅 httpOnly cookie；CSRF 自实现；登录失败 IP 限流 + 账号锁定。
9. **前端**：
   - 样式只写 `web/src/styles.css`（token + 组件），不引 CDN / 组件库
   - 状态一律 `StatusBadge`：字形 + 中文 + 颜色三重编码，颜色不单独承载语义
   - 破坏性操作走 `ConfirmDialog`，反馈走 `Toast`，不用 `window.confirm/prompt`
   - 改前端后必须重新 `cargo build` 才会嵌入新 `dist`
10. **数据库**：备份用 `sqlite3 .backup`，不用 `cp`；`busy_timeout=5000`。

## 环境变量

见 `.env.example`。`STEELPIPE_JWT_SECRET` 为空或短于 32 字节时拒绝启动。
本地开发凭据放 `dev.env`（已 gitignore，**不要提交**）。

## 测试重点

审计链黄金向量、TOCTOU 守卫、批量出库原子性、盘盈合规门、登录锁定审计、软删可见性、`used_count` 漂移修复、排序白名单与分页稳定性。
