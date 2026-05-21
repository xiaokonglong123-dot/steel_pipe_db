# Steel Pipe DB — 项目索引

## 架构概览

Rust + React 19 双包单体仓库。钢管制造的库存管理系统。

```
steel-pipe-db/
├── backend/          ← Rust Axum REST API（SQLite, JWT）
│   └── src/
│       ├── handlers/     ← HTTP 层：校验 → 调用服务 → 响应
│       ├── services/     ← 业务逻辑、编排
│       ├── repositories/ ← SQL 查询（SQLx）
│       ├── models/       ← 数据库行结构体
│       ├── dto/          ← 请求/响应类型
│       ├── domain/       ← 枚举、领域类型
│       └── middleware/   ← 认证、RBAC
├── frontend/         ← React 19 + Vite + Ant Design + TanStack Query
│   └── src/
│       ├── features/    ← [pipes|inventory|purchases|reports|production|customers]
│       │                   ├── api/       ← TanStack Query 钩子
│       │                   ├── hooks/     ← 特性相关的 React 钩子
│       │                   ├── pages/     ← 路由页面组件
│       │                   └── types/     ← TypeScript 接口
│       ├── lib/         ← validateResponse.ts, 运行时 zod 响应校验
│       └── zod-schemas/ ← 7 个 Zod Schema 文件，用于响应校验
└── docs/             ← 设计文档、ADR、任务分解
```

## 构建与运行

| 命令 | 说明 |
|---------|------|
| `make backend` | `cd backend && cargo build` |
| `make frontend` | `cd frontend && npm install && npm run build` |
| `make dev` | 同时启动后端（cargo run）和前端（开发服务器） |
| `make clean` | 清理所有构建产物 |
| `make test` | 运行所有测试（后端 + 前端） |
| `make run` | 生产环境完整构建 |
| `make build` | 构建两个包 |
| 前端 Chunk 分析 | `cd frontend && npx vite build --analyze`（通过 vite.config.ts 的 manualChunks） |

后端运行在 `http://localhost:3000`，前端开发服务器在 `http://localhost:5173`。

## 技术栈

### 后端
- **Rust** — nightly 工具链（2024-02-08），2021 版次
- **Axum** 0.7 — HTTP 框架
- **SQLx** 0.8 — 异步 SQL，编译时检查
- **SQLite** — 数据库（无需外部数据库服务）
- **JWT**（jsonwebtoken）— 认证令牌
- **bcrypt** — 密码哈希
- **serde** — JSON 序列化
- **tokio** — 异步运行时
- **tower-http** — CORS、日志中间件
- **依赖**：Backpack（校验器）、rust_decimal、bigdecimal、chrono

### 前端
- **React 19** — UI 框架
- **Vite** — 构建工具
- **TypeScript** — 类型安全
- **Ant Design 5** — UI 组件库
- **TanStack Query（React Query）** — 服务端状态管理
- **react-router-dom v7** — 路由
- **axios** — HTTP 客户端
- **i18next / react-i18next** — 国际化（主要语言 zh-CN）
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
- RBAC 作用域：`admin`、`manager`、`user`、`operator`
- 绝不压制类型错误（`as any`、`// @ts-ignore`、`// @ts-expect-error`）
- SQLx 编译时检查的查询（无原始 SQL 字符串）
- 所有小数值使用 `rust_decimal` / `BigDecimal` 类型

## 关键入口点
- **路由**：`backend/src/bin/main.rs` → 在 `/api/v1/{entity}` 挂载所有处理器
- **应用**：`frontend/src/App.tsx` → React Router 设置及布局
- **构建**：根目录 `Makefile` 编排两个包

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
