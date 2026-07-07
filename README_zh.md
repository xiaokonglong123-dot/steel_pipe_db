<div align="center">

> **🤖 本仓库中的所有代码均由 AI 生成** — 从架构设计到每一行代码，完全由大语言模型生成，用于技术演示和能力验证。

</div>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/API-5CT-1f2937?style=flat-square&logo=rust&logoColor=white">
  <img alt="API 5CT" src="https://img.shields.io/badge/API-5CT-1f2937?style=flat-square&logo=rust&logoColor=white">
</picture>

# Steel Pipe DB — API 5CT 无缝钢管 & 筛管 库存管理系统

> 油气行业 API 5CT 无缝钢管与筛管库存管理。Rust 后端，React 前端。名副其实。

![Rust](https://img.shields.io/badge/Rust-Axum-000000?style=flat-square&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-003B57?style=flat-square&logo=sqlite&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-5-3178C6?style=flat-square&logo=typescript&logoColor=white)
![Ant Design](https://img.shields.io/badge/Ant_Design-5-1677FF?style=flat-square&logo=antdesign&logoColor=white)

---

## 🚀 快速开始

### 前置条件

| 工具  | 版本      |
|-------|-----------|
| Rust  | 1.78+（edition 2021） |
| Node  | 20+       |
| npm   | 10+       |

### 后端

```bash
cd backend
cp .env.example .env    # 或自行配置：DATABASE_URL=sqlite://./data/steel_pipe.db?mode=rwc
cargo run               # 启动于 http://localhost:3000
```

### 前端

```bash
cd frontend
npm install
npm run dev             # 启动于 http://localhost:5173
```

打开 `http://localhost:5173`，使用以下账号登录：

| 用户名   | 密码       |
|----------|-----------|
| `admin`  | `admin123` |

---

## 🏗 技术栈

### 后端 — Rust (Axum 0.8)

| 层级         | 技术                                                |
|-------------|-----------------------------------------------------|
| 框架         | Axum 0.8（macros + multipart 特性）                  |
| ORM         | SQLx 0.8（SQLite, runtime-tokio-rustls）             |
| 认证         | JWT（jsonwebtoken 9）+ Argon2 密码哈希               |
| 校验         | Validator 0.19（derive 特性）                        |
| 日志         | Tracing + tracing-subscriber（env-filter, json）      |
| Excel/CSV   | calamine（导入）、rust_xlsxwriter（导出）、csv         |
| 中间件       | tower-http（CORS、trace、request-id）                 |

**架构：** Handler → Service → Repository → Domain。不使用 AppState — 数据库连接池通过 `Extension<SqlitePool>` 注入，认证密钥使用脱敏的 `JwtSecret` 扩展。

### 前端 — React 19

| 类别          | 库                                                |
|--------------|---------------------------------------------------|
| UI 框架       | React 19 + Ant Design 5 + @ant-design/icons        |
| 路由          | react-router-dom 7                                 |
| 状态管理       | Zustand 5（客户端状态）+ TanStack Query 5（服务端状态） |
| HTTP 客户端   | Axios                                              |
| 国际化        | react-i18next + i18next（中英文按模块划分）           |
| 构建工具       | Vite 6                                             |
| 类型安全       | TypeScript 5 + Zod 3                               |

---

## 📚 功能模块

### 阶段一 — 核心（P0）
| 模块       | 描述                                           |
|-----------|------------------------------------------------|
| 认证       | JWT 登录/刷新/登出，RBAC（admin/warehouse/qc/sales 四种角色） |
| 钢管       | API 5CT 钢管主数据（钢级、热处理、螺纹）            |
| 库存       | 逐根钢管追踪、ATP 可用量计算、库存日志、入库模板（从采购单自动填充）、出库库存感知（浏览在库钢管） |

### 阶段二 — 业务（P1）
| 模块       | 描述                                           |
|-----------|------------------------------------------------|
| 供应商     | 供应商管理、资质追踪                              |
| 客户       | 客户管理、信用/合同历史                            |
| 采购       | 采购单管理、入库审批流程                            |
| 销售       | 销售订单、出库、自动 ATP 检查                      |
| 质检       | 检验证书、无损检测、力学性能测试                    |
| 数据导入导出 | Excel/CSV 批量导入与导出                          |

### 阶段三 — 企业级（P2）
| 模块       | 描述                                           |
|-----------|------------------------------------------------|
| 合同       | 销售/采购合同、付款里程碑                          |
| 报表       | 仪表盘、日报/月报/统计报表                         |
| 标签       | 条码和规格标签生成                                 |
| 国际化     | 中英文切换、公制/英制单位                           |

---

## 🗄 数据模型

SQLite 中共 19 张表（WAL 模式，无外键约束 — 完整性在应用层保障，因为 SQLite 的 FK 支持不太行）：

```
pipes                → 钢管主数据（API 5CT 规格）
inventory            → 按钢管规格的当前库存
inventory_logs       → 逐根钢管的流转审计日志
suppliers            → 供应商主数据
customers            → 客户主数据
purchase_orders      → 采购单头
purchase_order_items → 采购单行项目
sales_orders         → 销售订单头
sales_order_items    → 销售订单行项目
inbound_records      → 入库单头（采购、生产、退货、调拨）
inbound_record_items → 入库单行项目
outbound_records     → 出库单头（销售、报废、调拨）
outbound_record_items→ 出库单行项目
quality_certificates → 质检证书
quality_mechanical   → 力学性能测试结果
quality_ndt          → 无损检测结果（UT/MI/MPI）
contracts            → 合同头
contract_milestones  → 付款/交付里程碑
users                → 系统用户（4 种角色）
```

所有时间戳均为 ISO 8601 字符串。通过 `deleted_at` 实现软删除 — 数据永不真正消亡。

---

## 🧪 开发

```bash
# 后端
cd backend && cargo check           # 仅类型检查（比完整构建快得多）
cargo test                           # 运行测试
cargo build                          # Debug 构建
cargo build --release                # 发布构建

# 前端
cd frontend && npx tsc --noEmit     # TypeScript 类型检查
npm run build                        # 生产构建
npm run lint                         # ESLint 检查
```

---

## 🔐 安全

- **密码**：Argon2id，推荐参数（`m=19456, t=2, p=1`）
- **认证**：JWT 可配置过期时间，刷新令牌轮换
- **RBAC**：4 种角色 — `admin`、`warehouse`、`qc`、`sales` — 通过中间件强制执行
- **数据**：所有业务实体软删除，通过 `inventory_logs` 审计追踪

---

## 📁 项目结构

```
steel_pipe_db/
├── backend/
│   ├── src/
│   │   ├── main.rs           # 入口，服务器启动
│   │   ├── lib.rs             # 模块声明
│   │   ├── router.rs          # 路由定义（约 70 个端点）
│   │   ├── config.rs          # 环境配置
│   │   ├── error.rs           # AppError 与 ApiResponse 映射；ApiErrorResponse 包含 success + request_id
│   │   ├── response.rs        # ApiResponse<T> / PaginatedResponse<T> / Meta 结构体，含 request_id（uuid v4）
│   │   ├── domain/            # 领域枚举和常量（钢管规格等）
│   │   ├── dto/               # 请求/响应 DTO
│   │   ├── models/            # 数据库模型（19 张表）
│   │   ├── repositories/      # SQL 查询层
│   │   ├── services/          # 业务逻辑层
│   │   ├── handlers/          # Axum 请求处理器
│   │   └── middleware/        # 认证 + RBAC 中间件
│   ├── migrations/            # SQLx 迁移文件
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   │   ├── api/               # Axios API 客户端
│   │   ├── features/          # 按模块划分：auth、pipes、inventory、purchases...
│   │   ├── layouts/           # MainLayout 含侧边栏
│   │   ├── stores/            # Zustand 状态仓库
│   │   ├── routes/            # react-router 路由配置
│   │   ├── shared/            # 共享组件和 hooks
│   │   ├── i18n/              # 中英文语言包
│   │   ├── types/             # 全局 TypeScript 类型
│   │   └── styles/            # 全局样式
│   ├── package.json
│   └── vite.config.ts
├── docs/                      # 设计与运维文档
│   ├── requirements.en.md     # 产品需求文档（英文）
│   ├── detailed-design.en.md  # 架构 + 数据库 + API 设计（英文）
│   ├── frontend-design.en.md  # 前端组件树 & 路由（英文）
│   ├── 需求文档.md             # 产品需求文档（中文）
│   ├── 详细设计文档.md         # 架构 + 数据库 + API 设计（中文）
│   ├── 前端设计文档.md         # 前端设计（中文）
│   ├── deployment.md          # 部署指南（Nginx、Docker、备份）
│   ├── troubleshooting.md     # 故障排查（数据库锁定、JWT、CORS）
│   └── tasks/                 # 任务分解（约 320 项）
└── .github/workflows/
    └── ci.yml                 # CI：cargo check + tsc + vite build
```

---

## 🌐 API 概览

所有端点位于 `/api/v1/` 下：

| 分组         | 前缀                  | 需要认证 |
|-------------|----------------------|:---:|
| 认证         | `/auth/*`            | 部分需要 |
| 用户         | `/users/*`           | 仅管理员 |
| 钢管         | `/pipes/*`           | 是 |
| 库存         | `/inventory/*`       | 是 |
| 供应商       | `/suppliers/*`       | 是 |
| 客户         | `/customers/*`       | 是 |
| 采购         | `/purchase-orders/*` | 是 |
| 销售         | `/sales-orders/*`    | 是 |
| 质检         | `/quality/*`         | 是 |
| 合同         | `/contracts/*`       | 是 |
| 报表         | `/reports/*`         | 是 |
| 标签         | `/labels/*`          | 是 |
| 数据导入导出  | `/data/*`            | 是 |

每个响应遵循统一格式：
```json
{ "success": true, "request_id": "req_...", "data": { ... } }
```
分页响应额外包含 `meta: { total, page, page_size, total_pages }`。错误响应将 `success` 置为 `false`，同样包含 `request_id`。

---

## 🔑 RBAC 权限矩阵

| API 分组         | admin | warehouse | qc  | sales |
|-----------------|:-----:|:---------:|:---:|:-----:|
| 用户（写入）      | ✅    | ❌        | ❌  | ❌    |
| 钢管（写入）      | ✅    | ✅        | ❌  | ❌    |
| 入库/出库（写入）  | ✅    | ✅        | ❌  | ❌    |
| 质检（写入）      | ✅    | ❌        | ✅  | ❌    |
| 销售订单（写入）   | ✅    | ❌        | ❌  | ✅    |
| 采购订单（写入）   | ✅    | ✅        | ❌  | ✅    |
| 供应商/客户（写入） | ✅    | ✅        | ❌  | ✅    |
| 合同（写入）      | ✅    | ✅        | ❌  | ✅    |
| 数据导入          | ✅    | ❌        | ❌  | ❌    |
| 标签（写入）      | ✅    | ✅        | ❌  | ❌    |
| 所有读取端点      | ✅    | ✅        | ✅  | ✅    |

---

## 🧭 设计文档

设计文档位于 [`docs/`](./docs/) 目录：

| 文档                    | 内容                                    |
|------------------------|-----------------------------------------|
| `需求文档.md`            | 完整产品需求：功能、API 5CT 标准、路线图    |
| `详细设计文档.md`        | 架构、19 表数据库设计、REST API、安全方案   |
| `前端设计文档.md`        | 组件树、路由、状态管理、国际化、主题        |
| `detailed-design.en.md` | 详细设计文档英文版                        |
| `requirements.en.md`    | 需求文档英文版                           |
| `frontend-design.en.md` | 前端设计文档英文版                        |
| `deployment.md`         | 部署指南：生产配置、Nginx、Docker、备份    |
| `troubleshooting.md`    | 故障排查：数据库锁定、JWT、CORS、迁移      |
| `tasks/progress.md`     | 主任务追踪（3 个阶段约 320 项）           |

另见：[`CONTRIBUTING.md`](../CONTRIBUTING.md) · [`CHANGELOG.md`](../CHANGELOG.md)

---

## 📄 许可证

[GNU General Public License v2](./LICENSE)

