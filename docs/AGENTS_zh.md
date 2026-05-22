# `docs/` — 设计文档与架构决策

## 结构

```
docs/
├── AGENTS.md              ← 英文版
├── AGENTS_zh.md           ← 本文档
├── 需求文档.md             ← 产品需求文档（中文）
├── 详细设计文档.md          ← 架构与设计（中文）
├── 前端设计文档.md           ← 前端设计（中文）
├── requirements.en.md     ← 产品需求文档（英文）
├── detailed-design.en.md  ← 详细设计（英文）
├── frontend-design.en.md  ← 前端设计（英文）
├── tasks/                 ← 任务分解
│   ├── progress.md
│   ├── phase1/            ← 认证、钢管、库存
│   ├── phase2/            ← 业务功能
│   └── phase3/            ← 企业功能
└── superpowers/           ← 架构规范
    └── specs/
```

## 架构决策

### 为什么选择 SQLite？
- 生产环境无需外部数据库服务器
- 单文件数据库，易于备份和部署
- SQLx 编译时查询检查可在构建时捕获 SQL 错误
- 足以应对单仓库/多仓库规模

### 为什么选择 Rust + React？
- **Rust**：类型安全、报表生成和库存计算的性能优势、无 GC 开销的内存安全。Axum 提供了友好的异步 handler。
- **React 19**：成熟的生态系统，Ant Design 提供企业级 UI 组件，TanStack Query 简化了服务端状态管理。

### 为什么选择基于特性的前端？
- 每个特性（钢管、库存、采购等）是自包含的
- 清晰的边界防止跨模块耦合
- 可并行开发（不同智能体负责不同特性）
- 添加/移除特性时无需触及无关代码

### 单体仓库 vs 独立仓库
- 单一仓库便于协调版本
- 直接使用 cargo/npm 命令，每个包独立运行
- 后端从内嵌静态文件提供前端构建产物；前端开发使用 Vite 代理到后端

## 决策记录

| 决策 | 选择 | 备选方案 | 理据 |
|----------|--------|-------------|-----------|
| 数据库 | SQLite | PostgreSQL | 部署更简单，规模够用 |
| HTTP 框架 | Axum 0.8 | Actix、Rocket | 生态系统、易用性、tower 生态 |
| ORM | SQLx | Diesel、SeaORM | 编译时 SQL 检查，无 ORM 开销 |
| UI 库 | Ant Design 5 | MUI、ShadCN | 企业导向、中文生态、表格质量 |
| 状态管理 | TanStack Query | Redux、Zustand | 服务端状态聚焦、缓存、去重 |
| 国际化 | i18next | react-intl、Lingui | 成熟生态、命名空间支持、懒加载 |
| 认证 | JWT + RBAC | Session 方案 | 无状态、移动端友好 |

## 关键设计文档
- `需求文档.md` — 产品需求文档（中文）
- `详细设计文档.md` — 架构与数据库设计（中文）
- `前端设计文档.md` — 前端组件树与路由（中文）
- `requirements.en.md` — 产品需求文档（英文）
- `detailed-design.en.md` — 架构与设计（英文）
- `frontend-design.en.md` — 前端设计（英文）
- `tasks/progress.md` — 各阶段主任务追踪

## 流程说明
- 文档是活文档——实施过程中如发现设计缺口，应及时更新
- AGENTS.md 文件是 AI 辅助开发的权威参考
- `docs/tasks/` 中的任务分解用于追踪实施状态
