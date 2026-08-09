# `docs/` — 设计文档与架构决策

这里是设计决策的存放地。并非所有内容都有书面记录，但重要的都有。

## 当前架构（ERP 重构）

> 历史沿革：本系统由钢管行业系统重构而来，已重构为通用 ERP（企业资源计划系统）。

- **项目**：ERP（通用企业资源计划系统）— 后端 crate 名为 `erp-server`（代码阶段的实施目标）
- **数据库**：SQLite3，连接串 `sqlite://data/erp.db?mode=rwc`，sqlx 0.8 `sqlite` 特性。旧版 37 个迁移文件将重写为 SQLite 语法，删除管材专属表。
- **保留模块**：auth/RBAC、workflow 审批、hr、finance、procurement、sales_crm、inventory（泛化为商品/Item+SKU：sku/名称/分类/单位/规格）、manufacturing、project、assets、notification、portal、bi、customers、suppliers、contracts、purchases、sales
- **删除模块**：管材主数据、标签打印、质检证书、行业参考数据，以及 search 与 data-io 中的管材逻辑
- **术语**：所有文档必须使用 [`specs/UBIQUITOUS_LANGUAGE_LATEST.md`](../specs/UBIQUITOUS_LANGUAGE_LATEST.md) 中的统一术语（商品/Item+SKU、采购订单、销售订单、质检/Inspection、工单等）

## 结构

```
docs/
├── AGENTS.md              ← 英文版
├── AGENTS_zh.md           ← 本文件
├── 需求文档.md             ← PRD（中文）
├── 详细设计文档.md          ← 架构与数据库设计（中文）
├── 前端设计文档.md           ← 前端设计（中文）
├── requirements.en.md     ← PRD（英文）
├── detailed-design.en.md  ← 详细设计（英文）
├── frontend-design.en.md  ← 前端设计（英文）
├── tasks/                 ← 任务分解
│   ├── progress.md
│   ├── phase1/            ← 认证、商品、库存、采购、销售
│   ├── phase2/            ← 业务功能
│   └── phase3/            ← 企业级功能
└── superpowers/           ← 架构规格
    └── specs/
```

## 架构决策

### 为什么用 SQLite？

- 生产环境无需安装或管理数据库服务器。
- 单文件存储 — 备份、部署都极其简单。
- SQLx 在编译期检查 SQL，错误提前暴露。
- 对单站点或多站点 ERP 规模绰绰有余。

### 为什么用 Rust + React？

- **Rust**：类型安全，报表与库存计算性能好，无 GC 的内存安全。Axum 让异步 handler 简洁直观。
- **React 19**：生态成熟。Ant Design 提供企业级 UI 组件。TanStack Query 解决服务端状态管理。

### 为什么按功能模块组织前端？

- 每个功能模块（商品、库存、采购等）自包含。
- 清晰的边界防止模块互相纠缠。
- 可以跨功能模块并行开发。
- 增删功能不会牵连无关代码。

### Monorepo 还是独立仓库？

- 单一仓库保证版本同步。
- 后端和前端各自有构建命令 — 无需 monorepo 工具链。
- 后端将构建后的前端作为内嵌静态文件提供。开发模式用 Vite 代理 API 请求到后端。

## 决策记录

| 决策         | 选择            | 备选方案            | 原因                          |
|-------------|----------------|--------------------|-------------------------------|
| 数据库       | SQLite3        | 客户端-服务器型 RDBMS | 部署简单，性能足够              |
| HTTP 框架   | Axum 0.8       | Actix、Rocket      | 易用性好，tower 中间件生态      |
| ORM         | SQLx           | Diesel、SeaORM     | 编译期 SQL 检查，开销最小       |
| UI 库       | Ant Design 5   | MUI、ShadCN        | 企业级、表格组件强、中文生态    |
| 状态管理     | TanStack Query | Redux、Zustand     | 专为服务端状态设计 — 缓存、去重、刷新 |
| 国际化       | i18next        | react-intl、Lingui | 成熟、命名空间、懒加载          |
| 认证         | JWT + RBAC     | 会话机制            | 无状态，兼容移动端             |

## 关键设计文档

- `需求文档.md` — 产品需求（中文）
- `详细设计文档.md` — 架构与数据库设计（中文）
- `前端设计文档.md` — 前端组件树与路由（中文）
- `requirements.en.md` — 产品需求（英文）
- `detailed-design.en.md` — 架构与设计（英文）
- `frontend-design.en.md` — 前端设计（英文）
- `tasks/progress.md` — 主任务追踪

## 流程说明

- 这些文档是活的 — 当实现揭示设计问题时及时更新。
- AGENTS.md 文件是 AI 辅助开发的规范来源。
- `docs/tasks/` 中的任务分解跟踪各阶段的实施状态。
