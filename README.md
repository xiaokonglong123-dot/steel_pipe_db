# Ayanami_Rei

API 5CT 无缝钢管与筛管库存管理系统。FastAPI + Rust (PyO3) + SQLite，Jinja2 服务端渲染，无前端构建链。

![Python](https://img.shields.io/badge/Python-3.11+-3776AB?style=flat-square&logo=python&logoColor=white)
![FastAPI](https://img.shields.io/badge/FastAPI-0.115+-009688?style=flat-square&logo=fastapi&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-PyO3-000000?style=flat-square&logo=rust&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-WAL-003B57?style=flat-square&logo=sqlite&logoColor=white)

---

## 功能

- **库存流转**：单管/批量入库、出库、调拨；CSV 批量导入（20MB 上限，逐行部分成功）
- **盘点**：盘点单持久化、差异应用（一致 / 缺失 / 位置异常 / 盘盈）、找回
- **API 5CT 合规**：标准规格 CRUD，入库前按钢级/外径/壁厚校验，盘盈同样强制合规
- **防篡改审计链**：SHA-256 链式哈希，`BEGIN IMMEDIATE` 同事务写入，20K 分块校验
- **RBAC**：`admin` / `operator` / `viewer` 三级角色
- **库位管理**：创建 / 重命名 / 停用，全部入审计链

## 技术栈

| 层 | 选型 |
|---|---|
| Web | FastAPI + Uvicorn（**必须单 worker**） |
| 模板 | Jinja2 SSR（`CompatTemplates` 兼容 Starlette 0.x/1.x） |
| 样式 | 本地 vendor Bootstrap 5.3.3 + 自有 token 层（`src/static/app.css`），无 CDN |
| 数据库 | SQLite（WAL，`BEGIN IMMEDIATE` 单写者） |
| ORM / 迁移 | SQLAlchemy 2.0 async + Alembic |
| 安全核心 | Rust PyO3：Argon2 密码哈希、JWT HS256、合规计算、审计哈希（全部纯函数、零状态） |

## 快速开始

```bash
# 1. 依赖
uv sync                      # 或 pip install -e ".[imports]"

# 2. 构建 Rust 扩展
uv run maturin develop --release

# 3. 配置
cat > .env <<'EOF'
JWT_SECRET_KEY=<至少 32 字符的随机串>
DATABASE_URL=sqlite+aiosqlite:///./data/inventory.db
ADMIN_USERNAME=admin
ADMIN_PASSWORD=<首次启动用的管理员密码>
EOF

# 4. 建表
uv run alembic upgrade head

# 5. 启动（单 worker 是硬性要求）
uv run uvicorn src.main:app --host 0.0.0.0 --port 8000 --workers 1
```

首次启动会从 `ADMIN_USERNAME` / `ADMIN_PASSWORD` 幂等创建管理员。**已存在同名用户时会重置其密码哈希**，所以首次登录后应移除这两个环境变量。

## 测试与检查

```bash
uv run --inexact pytest -m "not slow" -q   # 167 passed, 3 deselected
uv run --inexact pytest -m slow            # 20K 审计链压测
uv run --inexact ruff check                # 必须零报错
```

`--inexact` 是必需的：`rust_core` 由 maturin 安装、不在 `uv.lock` 中，普通 `uv sync` 会把它删掉。

## 项目结构

```
src/
├── main.py              # app factory、lifespan、异常处理器
├── config.py            # pydantic-settings
├── models.py            # SQLAlchemy ORM
├── security.py          # JWT cookie 认证、RBAC、CSRF、登录限流
├── modules/
│   ├── auth/            # 登录 / 登出 / refresh 轮换
│   ├── warehouse/       # 管子、库位、盘点、标准规格、审计
│   └── users/           # 用户 CRUD
├── templates/           # Jinja2，按模块分组；_macros.html 放共享徽章宏
└── static/              # app.css / app.js / vendor（Bootstrap + Icons）
rust-core/src/           # auth.rs / compliance.rs / audit.rs（纯函数）
migrations/versions/     # Alembic
docs/                    # 设计文档（权威：2026-07-13-api5ct-inventory-design.md）
```

## 关键约束

部署与改动前请先读 [`AGENTS.md`](AGENTS.md) 的 CONVENTIONS 与 ANTI-PATTERNS。最容易被忽略的几条：

- **`--workers 1`**：SQLite 单写者，多 worker 会写坏数据
- **所有写操作走 `BEGIN IMMEDIATE`**，审计行与业务操作同事务
- **TOCTOU 守卫**：`WHERE status='in_stock' AND quality_mark != 'scrapped'` + 校验 `rows_affected == 1`
- **软删除**：管子用 `deleted_at`、库位用 `is_active`，不做物理删除
- **备份用 `sqlite3 .backup`**，不要 `cp`
- **状态展示一律走 `_macros.html`**：图标 + 文字 + 颜色三重编码，颜色不得是唯一信息载体

## 文档

| 文档 | 内容 |
|---|---|
| [`docs/2026-07-13-api5ct-inventory-design.md`](docs/2026-07-13-api5ct-inventory-design.md) | 设计权威：schema、TOCTOU、审计链、API 总表、异常映射、运维脚本 |
| [`AGENTS.md`](AGENTS.md) | 工程约定、反模式、命令速查 |
| [`CHANGELOG.md`](CHANGELOG.md) | 版本变更 |
