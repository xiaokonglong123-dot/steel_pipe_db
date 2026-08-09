# Feature Modules

本目录包含 ERP（通用企业资源计划系统）前端的功能模块，每个模块遵循统一的目录结构和开发模式。后端 crate 为 `erp-server`（代码阶段目标）；后端数据层为 **SQLite3**（`sqlite://data/erp.db?mode=rwc`，sqlx 0.8 sqlite feature）。历史沿革：本系统由钢管行业系统重构而来，管材专属模块已由通用商品/SKU 与制造质检取代。

## 快速导航

| 模块 | 路由 | 功能 | 后端 API |
| ------ | ------ | ------ | ---------- |
| `auth/` | `/login`, `/system/users`, `/system/roles`, `/system/departments` | 登录、RBAC（用户/角色/部门） | `/api/v1/auth/*`, `/api/v1/users/*`, `/api/v1/roles/*`, `/api/v1/departments/*` |
| `inventory/` | `/inventory/inbound`, `/inventory/outbound`, `/inventory/stock`, `/inventory/locations`, `/inventory/check`, `/inventory/logs` | 商品/SKU 主数据、库存、出入库、库位、盘点 | `/api/v1/items/*`, `/api/v1/inbound-records/*`, `/api/v1/outbound-records/*`, `/api/v1/inventory/*`, `/api/v1/locations/*` |
| `inventory_atp/` | `/inventory/atp` | 库存预留（ATP） | `/api/v1/reservations/*` |
| `suppliers/` | `/suppliers/*` | 供应商管理 | `/api/v1/suppliers/*` |
| `customers/` | `/customers/*` | 客户管理 | `/api/v1/customers/*` |
| `purchases/` | `/purchases/*` | 采购订单、审批流 | `/api/v1/purchase-orders/*` |
| `sales/` | `/sales/*`, `/sales/atp` | 销售订单、ATP 检查 | `/api/v1/sales-orders/*` |
| `sales_crm/` | `/sales/crm` | 客户信用、发货 | `/api/v1/shipments/*`, `/api/v1/customer-credit/*` |
| `contracts/` | `/contracts/*` | 采购/销售合同、付款里程碑 | `/api/v1/contracts/*` |
| `manufacturing/` | `/manufacturing` | BOM、工单、质检（Inspection）、不合格品单 | `/api/v1/manufacturing/*` |
| `project/` | `/projects` | 项目、WBS、预算 | `/api/v1/projects/*` |
| `assets/` | `/assets` | 固定资产、折旧、处置 | `/api/v1/assets/*` |
| `hr/` | `/hr/employees`, `/hr/salaries` | 员工、考勤、薪资、劳动合同 | `/api/v1/employees/*`, `/api/v1/attendance/*`, `/api/v1/salaries/*`, `/api/v1/contracts/labor/*` |
| `finance/` | `/finance` | 会计科目、日记账、发票、付款、试算平衡 | `/api/v1/accounts/*`, `/api/v1/journal/*`, `/api/v1/invoices/*`, `/api/v1/payments/*`, `/api/v1/trial-balance` |
| `procurement/` | `/procurement` | 采购申请、采购收货、采购报价、供应商评分 | `/api/v1/requisitions/*`, `/api/v1/receipts/*`, `/api/v1/quotes/*`, `/api/v1/scorecard/*` |
| `workflow/` | `/workflow/definitions`, `/workflow/my-tasks` | 审批流定义/实例/任务 | `/api/v1/workflow/*` |
| `notification/` | `/notifications` | 通知收件箱、模板、偏好 | `/api/v1/notifications/*` |
| `portal/` | `/portal` | 门户账户（Party）、PO 确认、SO 确认 | `/api/v1/portal/*` |
| `reports/` | `/reports`, `/reports/dashboard`, `/reports/trends` | 明细报表、仪表盘 | `/api/v1/reports/*` |
| `bi/` | `/bi` | BI 分析看板 | `/api/v1/bi/*` |
| `search/` | `/search` | 全局商品搜索 | `/api/v1/search/*` |
| `data-io/` | `/data-io/import`, `/data-io/export`, `/data-io/logs` | 通用数据导入导出、操作日志 | `/api/v1/data-io/*` |
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
