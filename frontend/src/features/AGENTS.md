# `features/` — The Feature Module Pattern

All 25 feature modules follow the same layout. This doc is both a reference and a template for adding new ones. Frontend of the **ERP（通用企业资源计划系统）** monorepo — backend crate `erp-server`, backend data layer **SQLite3** (`sqlite://data/erp.db?mode=rwc`, sqlx 0.8 sqlite feature).

## Feature Module Structure

```
features/{feature}/
├── api/           ← TanStack Query hooks
│   └── index.ts   ← useQuery, useMutation
├── hooks/         ← Feature-specific React hooks (optional)
│   └── index.ts
├── queryKeys.ts   ← TanStack Query key factory for this feature
├── pages/         ← Page components (one file per route)
│   ├── ListPage.tsx
│   ├── FormPage.tsx       ← Create + Edit in one
│   └── DetailPage.tsx
├── stores/        ← Zustand stores (optional, for complex features)
│   └── index.ts
└── types/         ← TypeScript interfaces
    └── index.ts   ← Entity types, request/response shapes
```

## Existing Features

| Feature | Routes | What it does |
| --------- | -------- | ------------- |
| `auth/` | `/login`, `/system/users`, `/system/roles`, `/system/departments` | Login, RBAC (users/roles/departments), auth state via Zustand |
| `inventory/` | `/inventory/inbound`, `/inventory/outbound`, `/inventory/stock`, `/inventory/locations`, `/inventory/check`, `/inventory/logs` | 商品/Item + SKU master data, 库存 (in/out/stock/locations/count sessions) |
| `inventory_atp/` | `/inventory/atp` | 库存预留 (ATP) for orders |
| `suppliers/` | `/suppliers/*` | 供应商 management |
| `customers/` | `/customers/*` | 客户 management |
| `purchases/` | `/purchases/*` | 采购订单, 审批流 |
| `sales/` | `/sales/*`, `/sales/atp` | 销售订单, ATP check |
| `sales_crm/` | `/sales/crm` | 客户信用, 发货 (shipments) |
| `contracts/` | `/contracts/*` | 采购/销售合同, payment milestones |
| `manufacturing/` | `/manufacturing` | BOM, 工单, 质检 (Inspection), 不合格品单 |
| `project/` | `/projects` | 项目, WBS, 预算 |
| `assets/` | `/assets` | 固定资产 registration, depreciation, disposal |
| `hr/` | `/hr/employees`, `/hr/salaries` | 员工, 考勤, 薪资, 劳动合同 |
| `finance/` | `/finance` | 会计科目, 日记账, 发票, 付款, 试算平衡 |
| `procurement/` | `/procurement` | 采购申请, 采购收货, 采购报价, 供应商评分 |
| `workflow/` | `/workflow/definitions`, `/workflow/my-tasks` | 审批流定义/实例/任务 |
| `notification/` | `/notifications` | 通知 inbox, templates, preferences |
| `portal/` | `/portal` | 门户账户 (party), PO accept, SO ack |
| `reports/` | `/reports`, `/reports/dashboard`, `/reports/inventory`, `/reports/orders`, `/reports/trends` | 明细报表 |
| `bi/` | `/bi` | BI 分析 dashboard |
| `search/` | `/search` | Global search across items, inventory, orders |
| `data-io/` | `/data-io/import`, `/data-io/export`, `/data-io/logs` | Generic item/inventory data import/export, operation logs |
| `quality/` | `/quality/certs` | 质检证书 (quality certificates) |
| `labels/` | `/labels` | 标签打印 (label printing) |
| `profile/` | `/profile/settings` | User settings, password change |

## Template: `api/index.ts` (TanStack Query Hooks)

```ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/api'
import type { FeatureType, ListParams } from './types'
import { featureQueryKeys } from '../queryKeys'

export function useListFeature(params: ListParams) {
  return useQuery({
    queryKey: featureQueryKeys.list(params),
    queryFn: () => api.get('/feature', { params }).then(r => r.data),
  })
}

export function useCreateFeature() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (data: FeatureType) => api.post('/feature', data),
    onSuccess: () => qc.invalidateQueries({ queryKey: featureQueryKeys.all }),
  })
}
```

## Template: `types/index.ts`

```ts
export interface FeatureType {
  id: number
  name: string
  // ...
}

export interface ListParams {
  page?: number
  page_size?: number
  // filter fields
}
```

## Template: `pages/ListPage.tsx`

```tsx
import { Table, Button } from 'antd'
import { useListFeature } from '../api'
import { useTranslation } from 'react-i18next'

export default function ListPage() {
  const { t } = useTranslation('feature_name')
  const { data, isLoading } = useListFeature({ page: 1 })
  // Ant Design Table with columns
  // Actions: Create, Edit, Delete buttons
}
```

## API Connection

- All API calls go through the shared native fetch wrapper at `src/api/client.ts` (base URL: `/api/v1`).
- Query key convention: define a feature-local `queryKeys.ts` factory (for example `featureQueryKeys.all`, `.list(params)`, `.detail(id)`) and use it from hooks.
- Mutations invalidate the appropriate factory key on success so affected lists/details refetch.
- Some features use `lib/validateResponse.ts` with Zod schemas from `zod-schemas/` for runtime response validation.

## Adding a New Feature Module

1. Create `features/{new_feature}/` with subdirs: `api/`, `hooks/`, `pages/`, `stores/`, `types/`.
2. Add `queryKeys.ts`, then write TanStack Query hooks in `api/index.ts` using that factory.
3. Build page components in `pages/`.
4. Register the route in `src/routes/index.tsx`.
5. Add i18n keys in both `src/i18n/zh/{new_feature}.json` and `src/i18n/en/{new_feature}.json`.
6. Add a Zod response schema in `src/zod-schemas/` if you want runtime validation.

## Conventions

- `useFeatureQuery()` for list queries, `useFeatureQuery(id)` for detail.
- `useCreateFeature()`, `useUpdateFeature()`, `useDeleteFeature()` for mutations.
- Always invalidate list queries after successful mutations.
- Do not add inline `queryKey: [...]` literals in feature API modules; centralize them in `queryKeys.ts`.
- CRUD UI uses Ant Design Table + Form + Modal.
- Pages import API through `../api`, never directly from `@/api`.
