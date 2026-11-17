# SteelPipe

面向局域网的无缝钢管与筛管（API 5CT）单根级库存管理系统。**单个二进制**（内嵌前端）+ **SQLite**，无 Python、无 Node 运行时依赖。

| 项 | 值 |
| --- | --- |
| 后端 | Rust 2024 / axum 0.8 / sqlx 0.8 / SQLite（WAL） |
| 前端 | Vite 8 + React 19 + TypeScript（构建期内嵌进二进制） |
| 安全核心 | argon2id、HS256 JWT、SHA-256 审计哈希链 —— 全部同进程，无 FFI |
| 部署 | 一个 `steelpipe` 可执行文件 + 一个 `.db` 文件 |
| 测试 | 114 个（Rust 单元/集成 + HTTP e2e） |

## 功能

- **库存流转**：入库（API 5CT 五项校验）、单根/批量出库（原子）、调拨、软删；全部单事务
- **盘点**：盘点单持久化、逐根结论（一致/盘亏/位置异常/盘盈）、原子确认、找回
- **合规**：钢级精确匹配、壁厚区间、外径 ±1%、长度、筛管参数；不合规可管理员强制入库（留审计）
- **审计链**：`sha256(prev‖action‖detail‖ts)`，`BEGIN IMMEDIATE` 同事务写入，分批校验
- **RBAC**：`admin` / `operator` / `viewer`，写路径按路由组挂载
- **导入**：CSV 流式、行级部分成功、上限 10 000 行、BOM 容错
- **仪表盘**：KPI（在库/调拨/盘亏/问题管/今日出入库）+ 库位占用 + 最近审计
- **界面**：高密度暗色数据网格（字号 14px/行高 34px，顶栏可切密/适/宽三档），状态三重编码（字形+中文+颜色），表头点击排序（后端白名单），`⌘K` 命令面板
- **运维**：`audit verify` / `reconcile` / `seed-standards` 子命令，备份/恢复脚本

## 快速开始

```bash
# 1. 构建（首次会拉依赖）
cargo build --release --workspace --bins     # 产出 target/release/steelpipe
(cd web && npm ci && npm run build)          # 产出 web/dist（被 rust-embed 内嵌）
# 注意：改前端后必须重新 cargo build 才会嵌入新产物

# 2. 配置
export STEELPIPE_JWT_SECRET="$(openssl rand -base64 48)"
export STEELPIPE_DATABASE_URL="sqlite://data/steelpipe.db"
export STEELPIPE_BIND="127.0.0.1:8080"
# 首次引导管理员（同名用户存在则重置口令）
export STEELPIPE_ADMIN_USERNAME=root
export STEELPIPE_ADMIN_PASSWORD='请换成至少 8 位的口令'

# 3. 启动
./target/release/steelpipe serve
# → http://127.0.0.1:8080
```bash

完整环境变量见 `.env.example`。`STEELPIPE_JWT_SECRET` 为空或短于 32 字节时**拒绝启动**。

## 命令行

```bash
steelpipe serve                                  # 启动 HTTP 服务
steelpipe openapi > openapi.json                 # 导出 OpenAPI（前端类型生成的来源）
steelpipe audit verify [--batch 1000]            # 分批校验审计链，并比对链尾表
steelpipe reconcile                              # 检测并修复 used_count 漂移（写审计）
steelpipe seed-standards --path standards.csv    # 种子化标准规格（幂等）
# 全局 --database / STEELPIPE_DATABASE_URL 覆盖数据库地址
```bash

## 部署

```bash
see deploy/README.md      # systemd 单元、目录布局、升级与回滚
scripts/backup.sh         # sqlite3 .backup，日备 7 份 + 周快照 4 份
scripts/restore.sh        # 恢复前先保留现场，恢复后跑 integrity_check
```bash

单写者约束：**只起一个实例**。这是 SQLite 的文件级特性，也是审计链 `BEGIN IMMEDIATE` 串行化的前提。

## 项目结构

```text
crates/
  domain/        角色与业务枚举、管子状态机（无 IO）
  compliance/    API 5CT 五项校验（纯函数）
  audit/         哈希链计算与分批校验（纯函数）
  auth/          argon2id + HS256（Keys 显式持有，无全局状态）
  infra/         SQLite 连接池、PRAGMA、sqlx 迁移、Uow（写事务+审计）、仓储
  service/       用例：认证/用户/库存/目录/盘点/导入/运维
  api/           axum 路由、DTO、RFC 9457 错误、OpenAPI（不依赖 sqlx）
  server/        单二进制装配、CLI 子命令、rust-embed 静态资源
web/             Vite + React SPA（nginx 式 SPA 回退由 axum 兜底）
scripts/ci/      分层断言（依赖图即架构约束）
deploy/          systemd 单元
docs/            设计权威、重构参考、规格索引、里程碑记录
```text

分层由 CI 断言，不靠自觉：`crates/api` 直接依赖里出现 `sqlx` 或 `steelpipe-infra` 即失败。

## 测试

```bash
cargo test --workspace                  # 114 个
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/ci/layering.sh
(cd web && npm run build)               # 含 tsc --noEmit
```bash

测试覆盖重点：审计链黄金向量（与旧实现逐字节一致）、TOCTOU 守卫、批量原子性、盘盈合规门、登录锁定审计、软删可见性、`used_count` 漂移与修复。

## 文档

| 文档 | 内容 |
| --- | --- |
| [`docs/2026-07-13-api5ct-inventory-design.md`](docs/2026-07-13-api5ct-inventory-design.md) | 业务与架构的设计权威（v0.8.0） |
| [`docs/2026-11-17-rewrite-reference.md`](docs/2026-11-17-rewrite-reference.md) | 重写方案：选型、架构、里程碑、风险 |
| [`docs/spec/`](docs/spec/) | 从旧测试提取的行为规格（A–H 域，编号可追溯） |
| [`docs/milestones/`](docs/milestones/) | M0–M7 交付记录与实测结果 |
| [`docs/2026-11-17-ui-redesign.md`](docs/2026-11-17-ui-redesign.md) | 前端重设计：GitHub 调研、设计规范、验收 |

## 历史

本项目由 FastAPI + PyO3 + SQLite + Jinja2 的旧实现重写而来；旧栈代码已移除，
其行为规格保留在 `docs/spec/`，设计意图保留在设计文档中。
