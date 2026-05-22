# Steel Pipe DB — 项目索引

## 架构概览

Rust + React 19 双包单体仓库。钢管制造的库存管理系统。

```
steel-pipe-db/
├── backend/          ← Rust Axum 0.8 REST API（SQLite, JWT/Argon2）
│   └── src/
│       ├── handlers/     ← 13 个文件，HTTP 层：校验 → 调用服务 → 响应
│       ├── services/     ← 12 个文件，业务逻辑（unit struct，静态方法）
│       ├── repositories/ ← 13 个文件，纯 SQL 查询（SQLx）
│       ├── models/       ← 11 个文件，数据库行结构体
│       ├── dto/          ← 14 个文件，请求/响应类型
│       ├── domain/       ← 4 个文件，枚举、领域类型
│       ├── middleware/   ← 认证、RBAC（auth.rs + rbac.rs，含 success/request_id）
│       ├── router.rs     ← ~70 个端点
│       ├── config.rs     ← 基于环境变量的配置
│       ├── error.rs      ← AppError 枚举，数字错误码；ApiErrorResponse 包含 success + request_id
│       └── response.rs   ← ApiResponse<T>, PaginatedResponse<T>, Meta 结构体, request_id (uuid v4)
├── frontend/         ← React 19 + Vite + Ant Design + TanStack Query
│   └── src/
│       ├── main.tsx       ← React DOM 入口（含 import './i18n' 初始化）
│       ├── App.tsx        ← ConfigProvider + QueryClientProvider + RouterProvider
│       ├── features/      ← 11 个功能模块（auth, pipes, inventory, suppliers, customers, ...）
│       │                   ├── api/       ← TanStack Query 钩子
│       │                   ├── hooks/     ← 功能相关的 React 钩子
│       │                   ├── pages/     ← 路由页面组件
│       │                   └── types/     ← TypeScript 接口
│       ├── layouts/       ← MainLayout（侧边栏 + 搜索 + 个人设置）
│       ├── stores/        ← Zustand authStore, appStore（全局状态）, unitStore（单位转换）
│       ├── routes/        ← createBrowserRouter + ProtectedRoute
│       ├── lib/           ← validateResponse.ts，运行时 zod 响应校验
│       ├── shared/        ← hooks (useDebounce)，components/（9 个共享组件：ConfirmModal 等）
│       ├── zod-schemas/   ← 7 个 Zod Schema 文件，用于响应校验
│       └── i18n/          ← react-i18next（zh-CN 为主）；新增命名空间：inventory, pipes, profile 等
└── docs/             ← 设计文档、任务分解
```

## 构建与运行

**注意：项目中没有 Makefile。使用以下直接命令：**

| 命令 | 说明 |
|------|------|
| `cd backend && cargo check` | 后端类型检查（比构建快） |
| `cd backend && cargo test` | 后端测试 |
| `cd backend && cargo run` | 启动后端开发服务器 |
| `cd frontend && npm run dev` | 启动前端开发服务器 |
| `cd frontend && npx tsc --noEmit` | 前端类型检查 |
| `cd frontend && npm run build` | 前端生产构建 |
| 前端 Chunk 分析 | `cd frontend && npx vite build --analyze`（通过 vite.config.ts 的 manualChunks） |
| 完整 CI 流水线 | `cargo check` + `tsc --noEmit` + `npm run build`（并行） |

后端运行在 `http://localhost:3000`，前端开发服务器在 `http://localhost:5173`。

## 技术栈

### 后端
- **Rust** — nightly 工具链（2024-02-08），2021 版次
- **Axum 0.8** — HTTP 框架（宏 + multipart 功能）
- **SQLx 0.8** — 异步 SQL（SQLite，runtime-tokio-rustls，chrono）
- **SQLite** — 数据库（无需外部数据库服务）
- **JWT**（jsonwebtoken 9）— 认证令牌
- **Argon2 0.5** — 密码哈希（非 bcrypt）
- **validator 0.19** — 参数校验（derive）
- **tracing** + tracing-subscriber — 日志（env-filter, json）
- **tower-http 0.6** — CORS、trace、request-id 中间件
- **calamine 0.26** — Excel 导入，**rust_xlsxwriter 0.80** — Excel 导出，**csv 1.3** — CSV
- **没有 `rust_decimal` / `bigdecimal`** — 小数使用 f64 处理
- **没有 `build.rs`** — 尽管下级 AGENTS.md 中有所提及

### 前端
- **React 19** — UI 框架（react-router-dom v7，createBrowserRouter）
- **Vite** — 构建工具
- **TypeScript strict** — 类型安全（noUnusedLocals, noUnusedParameters）
- **Ant Design 5** — UI 组件库（+ @ant-design/icons）
- **TanStack Query 5** — 服务端状态管理（staleTime: 2min, gcTime: 5min）
- **Zustand 5** — 客户端状态管理
- **react-router-dom v7** — 路由
- **axios** — HTTP 客户端（/api/v1，自动附带 Bearer 令牌）
- **i18next / react-i18next** — 国际化（主要语言 zh-CN，按功能模块命名空间）
- **dayjs** — 日期处理
- **zod** — Schema 校验
- **zod 运行时校验** — `src/lib/validateResponse.ts` 封装 `zod.response()` 用于 API 响应校验

## 核心约定

### 后端分层模式
```
请求 → 处理器（校验）→ 服务（业务逻辑）→ 仓库（SQL）→ 数据库
                                                                              ↓
响应 ← 处理器（格式化） ← 服务（编排）     ← 仓库（查询结果） ←
```

- **Handlers**：提取参数，调用一个服务方法，返回 `ApiResponse<T>` | `ErrorResponse`
- **Services**：编排业务逻辑，跨实体操作，事务管理
- **Repositories**：纯 SQL，单实体 CRUD，不含业务逻辑
- **Models**：与数据库模式匹配的行结构体，无方法
- **DTOs**：请求校验结构体（serde::Deserialize + `validator`），响应结构体

### 前端特性模式
```
features/{feature}/
├── api/      ← TanStack Query 钩子（useQuery, useMutation）
├── hooks/    ← 特性相关的 React 钩子
├── pages/    ← 页面组件（1 个文件 = 1 条路由）
└── types/    ← TypeScript 接口
```

- **死代码清理**：domain/dto/error/response/repo 模块中移除了 26 个未使用项。`#![allow(dead_code)]` 保留在 crate 根以压制合法的误报。

## 安全规则
- 所有变更端点需要 JWT 认证（中间件强制执行）
- RBAC 角色：`admin`（管理员）、`warehouse`（仓库）、`qc`（质检）、`sales`（销售）
- 绝不压制类型错误（`as any`、`// @ts-ignore`、`// @ts-expect-error`）
- SQLx 编译时检查的查询（无原始 SQL 字符串）
- 小数使用 f64（非 rust_decimal / BigDecimal）

## 关键入口点
- **路由**：`backend/src/router.rs` → 在 `/api/v1/{entity}` 挂载所有处理器（~70 个端点）
- **应用**：`frontend/src/App.tsx` → ConfigProvider + QueryClientProvider + RouterProvider
- **构建**：直接使用 cargo/npm 命令（无 Makefile）

## 下级 AGENTS.md 文件
- `backend/AGENTS_zh.md` — Rust 包详情、构建、依赖
- `backend/src/AGENTS_zh.md` — 模块连接、路由注册
- `backend/src/handlers/AGENTS_zh.md` — 处理器模式
- `backend/src/services/AGENTS_zh.md` — 服务层约定
- `backend/src/repositories/AGENTS_zh.md` — 仓库/CRUD 模式
- `frontend/AGENTS_zh.md` — 前端包详情
- `frontend/src/AGENTS_zh.md` — 应用结构、共享基础设施
- `frontend/src/features/AGENTS_zh.md` — 特性模块模板
- `docs/AGENTS_zh.md` — 设计文档索引与架构决策
