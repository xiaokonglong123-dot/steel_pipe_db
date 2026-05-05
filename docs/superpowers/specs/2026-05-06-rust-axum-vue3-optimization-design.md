# 钢管原料进出入库管理系统 — 优化扩展设计

## 概述

对 `rust-axum-vue3` 子项目进行全面的优化扩展，涵盖架构重构、权限系统、前端技术栈升级、功能增强、UI/UX 提升及性能优化。

## 策略

采用**并行双轨制**：后端（Rust/Axum）与前端（Vue 3）同步开发，通过共享 API 契约保证兼容性。分三阶段迭代交付。

## 后端设计

### 模块拆分

将 `db.rs`（1079行）按业务领域拆分为独立模块：

```
server/src/
├── main.rs              # 路由定义 + 启动
├── db/
│   ├── mod.rs           # 数据库初始化、连接管理、migration
│   ├── pipes.rs         # 钢管 CRUD 查询
│   ├── records.rs       # 出入库记录
│   ├── productions.rs   # 生产投料
│   ├── logs.rs          # 操作日志
│   └── reports.rs       # 报表统计查询
├── handlers/
│   ├── mod.rs        # 路由->处理器映射（所有 handler 重导出）
│   ├── pipes.rs      # 钢管 CRUD 处理器
│   ├── records.rs    # 出入库记录处理器
│   ├── productions.rs# 生产投料处理器
│   ├── auth.rs       # 认证相关处理器
│   ├── export.rs     # 导出处理器
│   ├── import.rs     # 导入处理器
│   └── system.rs     # 备份恢复 + 报表处理器
├── models.rs            # 数据模型 + 校验
├── auth.rs              # JWT 认证中间件 + 角色检查
├── error.rs             # AppError 枚举 + IntoResponse
└── types.rs             # 分页、排序等共享类型
```

拆分原则：每个模块对外暴露函数签名为 `pub fn query_(…) -> Result<…>`，内部使用 `rusqlite::Connection`，由上层 `mod.rs` 管理事务。

### 认证与授权

- **JWT**：使用 `jsonwebtoken` crate，HS256 签名，过期时间 24h
- **密码**：使用 `bcrypt` crate 哈希存储
- **三角色**：
  | 角色 | 权限 |
  |------|------|
  | `admin` | 全部操作：CRUD、导入导出、备份恢复、用户管理 |
  | `operator` | 日常操作：入库/出库/生产投料、查看数据 |
  | `viewer` | 只读：查看仪表盘、库存、记录、报表 |
- **Middleware**：Axum `FromRequestParts` 实现 `RequireAuth`，提取 token 验证并将用户信息注入请求扩展
- **路由保护**：在 `main.rs` 路由层使用 `axum::middleware::from_fn` 或分层 `Router`

### 新增接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/auth/login` | 登录，返回 JWT + 用户信息 |
| POST | `/api/auth/register` | 注册（仅 admin） |
| GET  | `/api/auth/me` | 获取当前用户信息 |
| PUT  | `/api/auth/password` | 修改密码 |

### 性能优化

- **连接池**：`r2d2 + r2d2_sqlite`，池大小 4-8
- **查询优化**：关键查询（仪表盘统计、库存列表、记录筛选）增加复合索引（`(status, material)`, `(material, entry_date)`, `(operation_type, operation_date)`），使用 `EXPLAIN QUERY PLAN` 验证索引命中
- **字典缓存**：材质/位置等字典数据使用 `once_cell` 或 `dashmap` 做内存缓存，定时刷新

### 测试

- 单元测试：`models.rs` 校验逻辑、`error.rs` 错误转换
- 集成测试：`tests/` 目录下对每个 db 模块进行 SQLite 内存数据库测试
- API 测试：使用 `axum::test` 进行路由级测试

### 新增 Cargo 依赖

```toml
jsonwebtoken = "9"
bcrypt = "0.15"
r2d2 = "0.8"
r2d2_sqlite = "0.24"
uuid = { version = "1", features = ["v4"] }
```

## 前端设计

### TypeScript 迁移

全部 `.js` / `.vue` 文件迁移至 TypeScript：
- 接口类型定义置于 `src/types/` 目录
- `.vue` 文件使用 `<script setup lang="ts">`
- `vite.config.ts` 启用 TypeScript 支持

### TypeScript 配置

- `tsconfig.json`：`strict: true`，`target: ES2022`，`module: ESNext`
- `tsconfig.node.json`：Vite 配置文件的 TS 支持
- `vue-tsc` 用于类型检查，集成到 `npm run typecheck` 脚本

### 类型定义

```
types/
├── pipe.ts       # SteelPipe, PipeQuery, PipeFormData
├── auth.ts       # User, UserRole, LoginRequest, LoginResponse
├── record.ts     # InventoryRecord, RecordQuery
├── production.ts # Production, ProductionFormData
├── stats.ts      # DashboardStats, MaterialStats, TrendData
└── common.ts     # PaginatedResponse, ApiError, DictData
```

### Pinia 状态管理

```
stores/
├── index.ts       # 聚合导出
├── auth.ts        # user, token, isAuthenticated, login(), logout(), checkAuth()
├── pipes.ts       # pipes[], query, pagination, fetchPipes(), createPipe(), …
├── records.ts     # records[], filters, fetchRecords()
├── production.ts  # productions[], fetchProductions(), createProduction()
└── ui.ts          # theme, sidebarCollapsed, notifications
```

`auth` store 关键行为：
- 初始化时从 `localStorage` 读取 token
- `login()` 调用 API → 存储 token + user → Axios 默认 header
- `logout()` 清除所有状态 → router 跳转 `/login`
- Axios 响应拦截器：401 → 自动 logout

### 路由与守卫

```
router/
├── index.ts       # 路由定义
└── guard.ts       # beforeEach: 认证检查 + 角色检查
```

路由元信息：
```typescript
meta: {
  requiresAuth: boolean
  roles?: UserRole[]
  title: string
}
```

### UI/UX 增强

**深色模式：**
- CSS 变量定义明/暗两套色值
- `useUIStore` 中 `theme: 'light' | 'dark'` 状态
- 监听 `prefers-color-scheme` 媒体查询
- 切换按钮在侧边栏底部
- 持久化到 `localStorage`

**响应式布局：**
- 断点：768px / 1024px / 1440px
- 侧边栏：≥1024px 固定展开，<1024px 抽屉式
- 表格：水平滚动 + 响应式列隐藏（xsmall 隐藏非关键列）
- 表单：栅格布局，小屏单列

**交互优化：**
- 表格行悬停高亮 + click 展开详情
- 批量操作：勾选 → 悬浮操作栏
- 删除确认：Modal 二次确认
- 操作反馈：全局 Toast 通知
- 加载状态：骨架屏（Skeleton）替代 spinner
- 空状态：插画 + 引导文字

**可视化（ECharts）：**
- 仪表盘：库存概览环图、近 7 日趋势折线图、材质分布柱状图
- 统计页：出入库对比柱状图、库存周转率

### 新增前端依赖

```json
{
  "dependencies": {
    "pinia": "^2.1",
    "echarts": "^5.5",
    "vue-echarts": "^6.6"
  },
  "devDependencies": {
    "typescript": "~5.6",
    "@vitejs/plugin-vue": "^6.0",
    "vue-tsc": "^2.1",
    "@types/node": "^22"
  }
}
```

### 组件目录重构

```
components/
├── common/
│   ├── AppButton.vue
│   ├── AppModal.vue
│   ├── AppTable.vue
│   ├── AppForm.vue
│   ├── AppSkeleton.vue
│   └── AppToast.vue
├── layout/
│   ├── AppSidebar.vue
│   ├── AppHeader.vue
│   └── AppLayout.vue
└── business/
    ├── PipeForm.vue
    ├── PipeTable.vue
    ├── RecordTable.vue
    └── ProductionForm.vue
```

## 执行计划

### 第1阶段：基础设施 + 核心新功能

| 后端 Track A | 前端 Track B |
|---|---|
| db.rs → db/mod.rs + pipes.rs + records.rs + productions.rs + logs.rs + reports.rs | TS 类型定义 + Pinia stores 搭建 |
| auth.rs: JWT 签发/验证中间件 + 用户表 migration | 登录页面 + 路由守卫 + auth store |
| handlers.rs 拆分 | API 层 ts 化 + Axios 拦截器 |
| 主路由添加 auth 保护层 | 基础 UI 组件 (Button, Modal, Table) |
| **交付：可登录系统，API 已模块化** | **交付：TS 化前端，路由受保护** |

### 第2阶段：功能叠加

| 后端 Track A | 前端 Track B |
|---|---|
| Rust 测试覆盖 (unit + integration) | ECharts 可视化 (仪表盘 + 统计页) |
| 查询优化 + 复合索引 | 深色模式 |
| 连接池 (r2d2) | 响应式布局适配 |
| 字典缓存 | 骨架屏 + 空状态 + Toast |
| **交付：性能提升，功能完整** | **交付：现代化 UI** |

### 第3阶段：打磨收尾

| 后端 Track A | 前端 Track B |
|---|---|
| CI 流水线 (Rust check + test) | 交互细节打磨 |
| 性能压力测试 | 集成联调 |
| 最终 Bug 修复 | 最终测试 |

## 不纳入范围

- WebSocket 实时推送（未来考虑）
- 移动端原生应用（未来考虑）
- 多语言国际化（当前仅有中文需求）
- Docker 容器化部署（可后续添加）
