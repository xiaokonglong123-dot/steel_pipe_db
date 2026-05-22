<div align="center">

> **🤖 本仓库全部代码由 AI 生成** — 从架构设计到每一行代码，均为大语言模型自动产出，仅做技术演示与能力验证用途。

</div>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/API-5CT-1f2937?style=flat-square&logo=rust&logoColor=white">
  <img alt="API 5CT" src="https://img.shields.io/badge/API-5CT-1f2937?style=flat-square&logo=rust&logoColor=white">
</picture>

# Steel Pipe DB — API 5CT 无缝钢管 & 筛管库存管理系统

> 面向石油天然气行业的 API 5CT 无缝钢管与筛管库存管理系统，以 Rust + React 构建。

![Rust](https://img.shields.io/badge/Rust-Axum-000000?style=flat-square&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-003B57?style=flat-square&logo=sqlite&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-5-3178C6?style=flat-square&logo=typescript&logoColor=white)
![Ant Design](https://img.shields.io/badge/Ant_Design-5-1677FF?style=flat-square&logo=antdesign&logoColor=white)

---

## 🚀 快速开始

### 环境要求

| 工具   | 版本             |
|--------|------------------|
| Rust   | 1.78+ (edition 2021) |
| Node   | 20+              |
| npm    | 10+              |

### 启动后端

```bash
cd backend
cp .env.example .env    # 或自行创建：DATABASE_URL=sqlite://./data/steel_pipe.db?mode=rwc
cargo run               # 启动在 http://localhost:3000
```

### 启动前端

```bash
cd frontend
npm install
npm run dev             # 启动在 http://localhost:5173
```

打开 `http://localhost:5173`，使用以下账号登录：

| 用户名    | 密码       |
|-----------|------------|
| `admin`   | `admin123` |

---

## 🏗 技术栈

### 后端 — Rust (Axum 0.8)

| 层           | 技术                                              |
|--------------|---------------------------------------------------|
| 框架         | Axum 0.8，支持宏 + multipart                      |
| ORM          | SQLx 0.8 (SQLite, runtime-tokio-rustls)           |
| 认证         | JWT (jsonwebtoken 9) + Argon2 密码哈希             |
| 参数校验     | Validator 0.19 (derive)                           |
| 日志         | Tracing + tracing-subscriber (env-filter, json)   |
| Excel/CSV    | calamine（导入）、rust_xlsxwriter（导出）、csv     |
| 中间件       | tower-http (CORS, trace, request-id)              |

**架构模式：** Handler → Service → Repository → Domain

### 前端 — React 19

| 类别         | 库                                                  |
|--------------|-----------------------------------------------------|
| UI 框架      | React 19 + Ant Design 5 + @ant-design/icons         |
| 路由         | react-router-dom 7                                  |
| 状态管理     | Zustand 5（客户端状态）+ TanStack Query 5（服务端状态）|
| HTTP 客户端  | Axios                                               |
| 国际化       | react-i18next + i18next（中英文按模块拆分）          |
| 构建工具     | Vite 6                                              |
| 类型安全     | TypeScript 5 + Zod 3                                |

---

## 📚 功能模块

### 阶段一 — 核心 (P0)
| 模块       | 描述                                              |
|------------|---------------------------------------------------|
| 认证       | JWT 登录/刷新/登出，RBAC（管理员/仓库/质检/销售）  |
| 钢管       | API 5CT 钢管主数据（钢级、热处理、螺纹）           |
| 库存       | 单支钢管粒度追踪、ATP 计算、库存日志、采购单自动填充入库模板、出库时浏览在库钢管 |

### 阶段二 — 业务 (P1)
| 模块       | 描述                                              |
|------------|---------------------------------------------------|
| 供应商     | 供应商管理、资质追踪                               |
| 客户       | 客户管理、信用/合同历史                            |
| 采购       | 采购单管理、入库审批流程                           |
| 销售       | 销售订单、出库、自动 ATP 校验                      |
| 质检       | 检验证书、无损检测、力学性能试验                   |
| 数据导入导出 | Excel/CSV 批量导入与导出                         |

### 阶段三 — 企业级 (P2)
| 模块       | 描述                                              |
|------------|---------------------------------------------------|
| 合同       | 销售/采购合同、付款里程碑                          |
| 报表       | 仪表盘、日报/月报/统计报表                         |
| 标签       | 条码与规格标签生成                                |
| 国际化     | 中英文国际化、公制/英制单位切换                    |

---

## 🗄 数据模型

SQLite 中 19 张表（WAL 模式，无外键约束——完整性在应用层保证）：

```
pipes                → 钢管主数据（API 5CT 规格）
inventory            → 按规格分类的当前库存
inventory_logs       → 单支钢管操作审计日志
suppliers            → 供应商主数据
customers            → 客户主数据
purchase_orders      → 采购单头
purchase_order_items → 采购单项
sales_orders         → 销售单头
sales_order_items    → 销售单项
inbound_records      → 入库记录头（采购、生产、退货、调拨）
inbound_record_items → 入库记录项
outbound_records     → 出库记录头（销售、报废、调拨）
outbound_record_items→ 出库记录项
quality_certificates → 质检证书
quality_mechanical   → 力学性能检测结果
quality_ndt          → 无损检测结果（UT/MI/MPI）
contracts            → 合同头
contract_milestones  → 付款/交付里程碑
users                → 系统用户（4 种角色）
```

所有时间戳为 ISO 8601 文本格式；通过 `deleted_at` 实现软删除。

---

## 🧪 开发命令

```bash
# 后端
cd backend && cargo check           # 仅类型检查（比构建快）
cargo test                           # 运行测试
cargo build                          # Debug 构建
cargo build --release                # Release 构建

# 前端
cd frontend && npx tsc --noEmit     # TypeScript 类型检查
npm run build                        # 生产构建
npm run lint                         # ESLint
```

---

## 🔐 安全

- **密码**：Argon2id，推荐参数 (`m=19456, t=2, p=1`)
- **认证**：JWT 可配置过期时间、刷新令牌轮换
- **RBAC**：4 种角色 — `admin`（管理员）、`warehouse`（仓库）、`qc`（质检）、`sales`（销售）— 通过中间件强制执行
- **数据**：所有业务实体支持软删除，通过 `inventory_logs` 实现审计追踪

---

## 📁 项目结构

```
steel_pipe_db/
├── backend/
│   ├── src/
│   │   ├── main.rs           # 入口点，服务启动
│   │   ├── lib.rs             # 应用状态，共享类型
│   │   ├── router.rs          # 路由定义（约 70 个端点）
│   │   ├── config.rs          # 环境配置
│   │   ├── error.rs           # AppError 与 ApiResponse 映射；ApiErrorResponse 含 success + request_id
│   │   ├── response.rs        # ApiResponse<T> / PaginatedResponse<T> / Meta 结构体，含 request_id（uuid v4）
│   │   ├── domain/            # 领域枚举与常量（钢管规格等）
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
│   │   ├── features/          # 按模块：auth、pipes、inventory、purchases...
│   │   ├── layouts/           # 带侧边栏的主布局
│   │   ├── stores/            # Zustand 状态仓库
│   │   ├── routes/            # react-router 路由配置
│   │   ├── shared/            # 共享组件与 Hook
│   │   ├── i18n/              # 中英文语言包
│   │   ├── types/             # 全局 TypeScript 类型
│   │   └── styles/            # 全局样式
│   ├── package.json
│   └── vite.config.ts
├── docs/                      # 设计文档（中文）
│   ├── 需求文档.md            # PRD
│   ├── 详细设计文档.md         # 架构 + 数据库 + API 设计
│   ├── 前端设计文档.md          # 前端组件树与路由
│   └── tasks/                 # 任务分解（约 320 项）
└── .github/workflows/
    └── ci.yml                 # CI：cargo check + tsc + vite build
```

---

## 🌐 API 概览

所有端点位于 `/api/v1/` 下：

| 分组       | 前缀                 | 需要认证 |
|------------|----------------------|:---:|
| 认证       | `/auth/*`            | 混合 |
| 用户       | `/users/*`           | 仅管理员 |
| 钢管       | `/pipes/*`           | 是 |
| 库存       | `/inventory/*`       | 是 |
| 供应商     | `/suppliers/*`       | 是 |
| 客户       | `/customers/*`       | 是 |
| 采购       | `/purchase-orders/*` | 是 |
| 销售       | `/sales-orders/*`    | 是 |
| 质检       | `/quality/*`         | 是 |
| 合同       | `/contracts/*`       | 是 |
| 报表       | `/reports/*`         | 是 |
| 标签       | `/labels/*`          | 是 |
| 数据导入导出 | `/data/*`          | 是 |

所有响应格式：
```json
{ "success": true, "request_id": "req_...", "data": { ... } }
```
分页响应包含 `meta: { total, page, page_size, total_pages }`。错误响应包含 `success: false` 和 `request_id`。

---

## 🧭 设计文档

设计文档（中文）位于 [`docs/`](./docs/)：

| 文档                   | 内容                                |
|------------------------|-------------------------------------|
| `需求文档.md`           | 完整 PRD：功能、API 5CT 标准、路线图 |
| `详细设计文档.md`        | 架构、19 表数据库设计、REST API、安全 |
| `前端设计文档.md`        | 组件树、路由、状态管理、国际化、主题   |
| `tasks/progress.md`     | 主任务追踪（~320 项，横跨 3 个阶段） |

---

## 📄 许可证

[GNU General Public License v2](./LICENSE)
