# 017 — ERP: 前端设计指南

> **版本**: v2.0（重构）
> **日期**: 2026-08-02
> **状态**: Draft
> **依赖**: `015-architecture-overview.md`
> **关联**: 所有模块 spec 的前端部分

---

## 目录

1. [技术栈延续](#1-技术栈延续)
2. [全局架构改造](#2-全局架构改造)
3. [路由与导航重新设计](#3-路由与导航重新设计)
4. [权限控制 UI](#4-权限控制-ui)
5. [实时通知系统](#5-实时通知系统)
6. [共享组件增强](#6-共享组件增强)
7. [移动端适配](#7-移动端适配)
8. [新模块集成清单](#8-新模块集成清单)

---

## 1. 技术栈延续

| 组件 | 当前 | ERP 升级 | 说明 |
| ------ | ------ | --------- | ------ |
| 框架 | React 19 | React 19 | 不变 |
| UI 库 | Ant Design 5 | Ant Design 5 | 不变 |
| 路由 | react-router-dom v7 | 同 | 不变 |
| 服务端状态 | TanStack Query 5 | TanStack Query 5 | 不变 |
| 客户端状态 | Zustand 5 | Zustand 5 | 不变 |
| HTTP 客户端 | Axios (apiClient.ts) | Axios | 不变 |
| 类型检查 | TypeScript 5 strict | TypeScript 5 strict | 不变 |
| 构建 | Vite | Vite | 不变 |
| 国际化 | react-i18next | react-i18next | 不变 |
| 运行时验证 | Zod | Zod | 不变 |
| 实时通知 | — | WebSocket | 新增 |

---

## 2. 全局架构改造

### 2.1 新增共享 Store — `notificationStore`

```typescript
// frontend/src/stores/notificationStore.ts
import { create } from 'zustand';

interface NotificationState {
  unreadCount: number;
  setUnreadCount: (n: number) => void;
  incrementUnread: () => void;
  decrementUnread: (n: number) => void;
}
```

### 2.2 新增权限控制组件 `Can`

```tsx
// frontend/src/shared/components/Can.tsx
interface CanProps {
  permissions: string[];
  children: React.ReactNode;
  fallback?: React.ReactNode; // null 表示隐藏
}

export function Can({ permissions, children, fallback = null }: CanProps) {
  const user = useAuthStore((s) => s.user);
  if (!user) return null;
  const hasAny = permissions.some((p) => user.permissions?.includes(p));
  return hasAny ? <>{children}</> : <>{fallback}</>;
}
```

### 2.3 新增 `useErrorMessage` hook

```ts
// frontend/src/shared/hooks/useErrorMessage.ts
export function useErrorMessage() {
  return useCallback((error: AxiosError | Error) => {
    if (isAxiosError(error)) {
      const code = error.response?.data?.code;
      const msg = error.response?.data?.message;
      if (code) return message.error(`[${code}] ${msg || t('common.error')}`);
    }
    message.error(t('common.error'));
  }, []);
}
```

### 2.4 新增 `FormPageLayout` 共享组件

```tsx
// frontend/src/shared/components/FormPageLayout.tsx
interface FormPageLayoutProps {
  title: string;
  isEdit: boolean;
  onBack: () => void;
  loading: boolean;
  onSubmit: (values: any) => Promise<void>;
  children: React.ReactNode;
}
```

---

## 3. 路由与导航重新设计

### 3.1 完整路由表

```tsx
// frontend/src/routes/index.tsx
export const router = createBrowserRouter([
  // ======================== PUBLIC ========================
  { path: '/login', element: <LoginPage /> },

  // ======================== PROTECTED ========================
  {
    path: '/',
    element: <ProtectedRoute><MainLayout /></ProtectedRoute>,
    children: [
      { index: true, element: <Navigate to="/dashboard" replace /> },

      // ----- 仪表板 -----
      { path: 'dashboard', element: route(<DashboardPage />) },

      // ----- 通知中心 -----
      { path: 'notifications', element: route(<NotificationCenterPage />) },

      // ----- 工作流 (审批中心) -----
      { path: 'workflow/approvals', element: route(<WorkflowApprovalListPage />) },
      { path: 'workflow/designer', element: route(<WorkflowDesignerPage />), handle: { roles: ['admin'] } },

      // ----- 商品管理 -----
      { path: 'items', element: route(<ItemListPage />) },
      { path: 'items/new', element: route(<ItemFormPage key="new" />) },
      { path: 'items/:id', element: route(<ItemDetailPage />) },
      { path: 'items/:id/edit', element: route(<ItemFormPage key="edit" />) },

      // ----- 库存管理 -----
      { path: 'inventory/inbound', ... },
      { path: 'inventory/outbound', ... },
      { path: 'inventory/stock', ... },
      { path: 'inventory/locations', ... },
      { path: 'inventory/check', ... },
      { path: 'inventory/logs', ... },
      { path: 'inventory/atp', element: route(<AtpQueryPage />) },  // 新增

      // ----- 订单管理 -----
      { path: 'purchases', ... },
      { path: 'sales', ... },
      { path: 'contracts', ... },

      // ----- 供应商 & 客户 -----
      { path: 'suppliers', ... },
      { path: 'customers', ... },

      // ----- 制造管理 -----
      { path: 'manufacturing/bom', element: route(<BomListPage />) },
      { path: 'manufacturing/bom/:id', element: route(<BomDetailPage />) },
      { path: 'manufacturing/work-orders', element: route(<WorkOrderListPage />) },
      { path: 'manufacturing/work-orders/new', element: route(<WorkOrderFormPage key="new" />) },
      { path: 'manufacturing/work-orders/:id', element: route(<WorkOrderDetailPage />) },

      // ----- 项目管理 -----
      { path: 'projects', ... },

      // ----- 财务管理 -----
      { path: 'finance/chart-of-accounts', element: route(<ChartOfAccountsPage />) },
      { path: 'finance/journal-entries', element: route(<JournalEntryListPage />) },
      { path: 'finance/journal-entries/new', element: route(<JournalEntryFormPage key="new" />) },
      { path: 'finance/receivables', element: route(<ReceivablesPage />) },
      { path: 'finance/payables', element: route(<PayablesPage />) },
      { path: 'finance/invoices', element: route(<InvoicesPage />) },
      { path: 'finance/payments', element: route(<PaymentsPage />) },
      { path: 'finance/ledger', element: route(<GeneralLedgerPage />) },

      // ----- 人力资源管理 -----
      { path: 'hr/employees', element: route(<EmployeeListPage />) },
      { path: 'hr/departments', element: route(<DepartmentPage />) },
      { path: 'hr/attendance', element: route(<AttendancePage />) },
      { path: 'hr/salary', element: route(<SalaryPage />) },

      // ----- 固定资产 -----
      { path: 'assets/register', element: route(<AssetRegisterPage />) },
      { path: 'assets/depreciation', element: route(<DepreciationPage />) },

      // ----- 报表 & BI -----
      { path: 'reports', element: route(<ReportListPage />) },
      { path: 'reports/dashboard', element: route(<DashboardPage />) },
      { path: 'reports/finance', element: route(<FinanceReportPage />) },
      { path: 'reports/inventory', element: route(<InventoryReportPage />) },
      { path: 'reports/supply-chain', element: route(<SupplyChainReportPage />) },
      { path: 'reports/manufacturing', element: route(<ManufacturingReportPage />) },

      // ----- 系统设置 -----
      { path: 'settings/users', element: route(<UserManagementPage />), handle: { roles: ['admin'] } },
      { path: 'settings/data-io', element: route(<DataIoPage />) },
      { path: 'settings/system', element: route(<SystemSettingsPage />), handle: { roles: ['admin'] } },

      { path: 'profile/settings', element: route(<ProfileSettingsPage />) },

      { path: '*', element: <Navigate to="/" replace /> },
    ],
  },
]);
```

### 3.2 侧栏菜单结构

```
Dashboard          — 首页全景
Notification       — 通知中心 (带数字角标)
────────────────────────────
Workflow
  ├── Approvals (审批中心)
  └── Designer (流程图)

商品管理
  └── 商品列表

库存
  ├── 入库管理
  ├── 出库管理
  ├── 库存查询
  ├── ATP 查询 (新增)
  ├── 库位管理
  └── 盘点管理

订单
  ├── 采购订单
  ├── 销售订单
  └── 合同管理

供应商 & 客户
  ├── 供应商
  └── 客户

制造
  ├── BOM 与工艺
  ├── 生产工单
  └── 质检记录

项目
  └── 项目列表

────────────────────────────
财务
  ├── 会计科目
  ├── 总账
  ├── 应收账款
  ├── 应付账款
  ├── 发票
  └── 收付款

人力资源
  ├── 员工管理
  ├── 部门管理
  ├── 考勤
  └── 薪资

固定资产
  ├── 资产登记
  └── 折旧

────────────────────────────
报表
  ├── 仪表板
  ├── 财务报表
  ├── 库存报表
  └── 供应链分析

设置
  ├── 用户管理
  ├── 数据导入导出
  └── 系统设置
```

### 3.3 导航重构原则

1. 原 sidebar 的模块合并为更大的业务类（商品 + 库存 + 订单 + 制造 + 供应链 + 财务 + 人力资源）。
2. Dashboard 设为根页面
3. 工作流独立模块
4. 安全管理 (`/system/users`) 改为 `/settings/users`
5. `/search` 作为 header 内的全局商品搜索框

---

## 4. 权限控制 UI

### 4.1 `Can` 组件使用示例

```tsx
// 采购订单页面 — 审批按钮
<Can permissions={['purchase.approve']}>
  <Button type="primary" onClick={handleApprove}>{t('common.approve')}</Button>
</Can>
```

### 4.2 权限清单

| 权限键 | 含义 |
| -------- | ------ |
| `item.read`, `item.write` | 商品数据 |
| `inventory.inbound`, `inventory.outbound` | 收发出入 |
| `purchase.read`, `purchase.approve`, `purchase.create` | 采购 |
| `sales.read`, `sales.approve`, `sales.create` | 销售 |
| `finance.read`, `finance.journal.post`, `finance.pay` | 财务 |
| `hr.read`, `hr.employee.write` | HR |
| `manufacturing.bom`, `manufacturing.work_order` | 制造 |
| `workflow.approver`, `workflow.design` | 工作流 |
| `system.admin` | 系统设置 |

---

## 5. 实时通知系统

### 5.1 WebSocket 对接

```typescript
// frontend/src/shared/hooks/useWebSocket.ts
export function useWebSocket() {
  const token = useAuthStore((s) => s.token);

  useEffect(() => {
    if (!token) return;
    const ws = new WebSocket(`wss://${location.host}/ws?token=${token}`);

    ws.onmessage = (event) => {
      const msg: WsMessage = JSON.parse(event.data);
      if (msg.type === 'notification') {
        notification.info({ message: msg.title, description: msg.body });
        useNotificationStore.getState().incrementUnread();
      }
    };

    return () => ws.close();
  }, [token]);
}
```

### 5.2 通知类型

| Type | 示例 |
| ------ | ------ |
| `workflow.assigned` | 有新的采购订单向您发出审批 |
| `workflow.approved` | 您的采购申请被批准 |
| `workflow.rejected` | 您的采购申请被驳回 |
| `inventory.item_changed` | 库存量变化 |
| `message` | 用户发送消息 |

---

## 6. 共享组件增强

当前 9 个共享组件保留，新增或增强：

| 组件 | 状态 | 描述 |
| ------ | ------ | ------ |
| `Can` | **新增** | 权限封装组件 |
| `FormPageLayout` | **新增** | 统一的表单页面模板 |
| `NotificationBell` | **新增** | 通知铃铛 + 计数角标 |
| `DataTable` | **增强** | 支持 exportable 行选择、列排序 |
| `PageLayout` | **不变** | 已足够好用 |
| `SearchBar` | **增强** | 支持跨功能区的自动建议 |
| `StatusTag` | **增强** | 增加制造、财务域颜色 |

---

## 7. 移动端适配

- 使用 Ant Design `Layout` 的 Breakpoint 响应应用，侧边栏可折叠
- DataTable 使用 `responsive: ['md']` 隐藏或缩减列
- 审批页面使用垂直表格式卡片在所有屏幕上一致

---

## 8. 新模块集成清单

当创建一个新的 ERP 模块（如 `finance`），按以下清单操作：

1. [ ] 在 `frontend/src/features/` 下创建新模块
2. [ ] 创建 `api/{module}Api.ts` — 使用 `apiClient` 对接 endpoint
3. [ ] 创建 `hooks/use{Module}.ts` — TanStack Query hooks
4. [ ] 创建 `types.ts` — 接口和请求类型
5. [ ] 创建 `queryKeys.ts` — key factory
6. [ ] 创建页面 pages（List、Form、Detail）
7. [ ] 在 `routes/index.tsx` 中添加路由
8. [ ] 在 `MainLayout.tsx` 侧边栏添加菜单项
9. [ ] 添加命名空间 i18n: `locales/zh/{module}.json` + `locales/en/{module}.json`
10. [ ] 在 `i18n/index.ts` 中加载 translations
11. [ ] 在 `zod-schemas/` 中定义任何新的 API 响应模式

---

> **关联**: `015-architecture-overview.md` · 每个子模块 spec 的前端部分均参照本指南
