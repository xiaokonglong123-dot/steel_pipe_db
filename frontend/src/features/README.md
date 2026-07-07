# Feature Modules

本目录包含 13 个业务功能模块，每个模块遵循统一的目录结构和开发模式。

## 快速导航

| 模块 | 路由 | 功能 | 后端 API |
|------|------|------|----------|
| `auth/` | (via ProtectedRoute) | 登录、认证状态管理 | `/api/v1/auth/*` |
| `pipes/` | `/pipes/seamless/*`, `/pipes/screen/*` | API 5CT 钢管主数据 | `/api/v1/seamless-pipes/*`, `/api/v1/screen-pipes/*` |
| `inventory/` | `/inventory/inbound`, `/inventory/outbound`, `/inventory/stock`, `/inventory/locations`, `/inventory/check` | 库存追踪、出入库、库位、盘点 | `/api/v1/inbound-records/*`, `/api/v1/outbound-records/*`, `/api/v1/inventory/*`, `/api/v1/locations/*` |
| `suppliers/` | `/suppliers/*` | 供应商管理 | `/api/v1/suppliers/*` |
| `customers/` | `/customers/*` | 客户管理 | `/api/v1/customers/*` |
| `purchases/` | `/purchases/*` | 采购单、审批流程 | `/api/v1/purchase-orders/*` |
| `sales/` | `/sales/*` | 销售订单、ATP 检查 | `/api/v1/sales-orders/*` |
| `quality/` | `/quality/certs/*` | 质检证书、力学/无损检测 | `/api/v1/quality/*` |
| `contracts/` | `/contracts/*` | 合同管理、付款里程碑 | `/api/v1/contracts/*` |
| `reports/` | `/reports`, `/reports/dashboard` | 报表、仪表盘 | `/api/v1/reports/*` |
| `labels/` | `/labels` | 条码标签生成 | `/api/v1/labels/*` |
| `search/` | `/search` | 全局搜索 | `/api/v1/pipes/search` |
| `profile/` | `/profile/settings` | 用户设置、修改密码 | `/api/v1/auth/me`, `/api/v1/auth/change-password` |

## 目录结构

```
features/{module}/
├── api/           ← TanStack Query hooks（useQuery, useMutation）
├── hooks/         ← 模块专用 React hooks（可选）
├── pages/         ← 页面组件（每个路由一个文件）
├── stores/        ← Zustand 状态（可选，仅复杂模块使用）
├── types/         ← TypeScript 接口定义
└── queryKeys.ts   ← TanStack Query key 工厂
```

## 开发新模块

1. 创建 `features/{module}/` 目录及子目录
2. 编写 `queryKeys.ts` 和 `api/index.ts`（TanStack Query hooks）
3. 在 `pages/` 中构建页面组件
4. 在 `src/routes/index.tsx` 注册路由
5. 在 `src/i18n/zh/` 和 `src/i18n/en/` 添加翻译
6. （可选）在 `src/zod-schemas/` 添加响应验证

详细规范请参考 [AGENTS.md](./AGENTS.md)。
