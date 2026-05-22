# `features/` — 特性模块模式

所有 13 个特性模块遵循相同的模式。本文档作为理解或添加任何特性的模板。

## 特性模块结构

```
features/{特性}/
├── api/           ← TanStack Query hooks（API 层）
│   └── index.ts   ← useQuery、useMutation hooks
├── hooks/         ← 特性专属 React hooks（可选）
│   └── index.ts
├── pages/         ← 页面组件（1 个文件 = 1 条路由）
│   ├── ListPage.tsx
│   ├── FormPage.tsx       ← 创建 + 编辑合并
│   └── DetailPage.tsx
├── stores/        ← Zustand 状态管理（可选，用于复杂特性）
│   └── index.ts
└── types/         ← TypeScript 接口
    └── index.ts   ← 实体类型、请求/响应类型
```

## 现有特性

| 特性 | 路由 | 描述 |
|---------|--------|-------------|
| `auth/` | (通过 ProtectedRoute) | 登录页面，认证状态管理（Zustand）|
| `pipes/` | `/pipes/seamless/*`, `/pipes/screen/*` | API 5CT 钢管主数据（无缝管 + 筛管）|
| `inventory/` | `/inventory/inbound`, `/inventory/outbound`, `/inventory/stock`, `/inventory/locations`, `/inventory/check` | 库存追踪、入库/出库、库位管理、盘点 |
| `suppliers/` | `/suppliers/*` | 供应商管理 |
| `customers/` | `/customers/*` | 客户管理 |
| `purchases/` | `/purchases/*` | 采购订单、审批流程 |
| `sales/` | `/sales/*` | 销售订单、ATP 检查 |
| `quality/` | `/quality/certs/*` | 质量证书、力学/NDT 检测 |
| `contracts/` | `/contracts/*` | 销售/采购合同、付款里程碑 |
| `reports/` | `/reports`, `/reports/dashboard` | 仪表板、日报/月报/统计报表 |
| `labels/` | `/labels` | 条码和规格标签生成 |
| `search/` | `/search` | 钢管、库存、订单全局搜索 |
| `profile/` | `/profile/settings` | 用户个人资料设置、修改密码 |

## 模板：`api/index.ts`（TanStack Query Hooks）
```ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/api'
import type { FeatureType, ListParams } from './types'

export function useListFeature(params: ListParams) {
  return useQuery({
    queryKey: ['feature', params],
    queryFn: () => api.get('/feature', { params }).then(r => r.data),
  })
}

export function useCreateFeature() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (data: FeatureType) => api.post('/feature', data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['feature'] }),
  })
}
```

## 模板：`types/index.ts`
```ts
export interface FeatureType {
  id: number
  name: string
  // ...
}

export interface ListParams {
  page?: number
  page_size?: number
  // 过滤字段
}
```

## 模板：`pages/ListPage.tsx`
```tsx
import { Table, Button } from 'antd'
import { useListFeature } from '../api'
import { useTranslation } from 'react-i18next'

export default function ListPage() {
  const { t } = useTranslation('feature_name')
  const { data, isLoading } = useListFeature({ page: 1 })
  // Ant Design Table 配合列定义
  // 操作：创建、编辑、删除按钮
}
```

## API 连接
- 所有 API 调用使用 `src/api/` 中的共享 axios 实例（基础 URL：`/api/v1`）
- 查询键遵循约定：列表使用 `['entity']`，详情使用 `['entity', id]`
- 变更操作成功后会失效列表查询键
- 部分特性 API 模块集成 `lib/validateResponse.ts`，通过 `zod-schemas/` 中的 Zod 模式进行运行时响应验证

## 添加新特性模块
1. 创建 `features/{new_feature}/`，包含 `api/`、`hooks/`、`pages/`、`stores/`、`types/` 子目录
2. 在 `api/index.ts` 中添加 TanStack Query hooks
3. 在 `pages/` 中添加页面组件
4. 在 `src/routes/index.tsx` 中添加路由
5. 在 `src/i18n/zh/{new_feature}.json` 和 `src/i18n/en/{new_feature}.json` 中添加 i18n 键
6. 如需 API 验证，在 `src/zod-schemas/` 中添加 Zod 响应模式

## 约定
- `useFeatureQuery()` 用于列表，`useFeatureQuery(id)` 用于详情
- `useCreateFeature()`、`useUpdateFeature()`、`useDeleteFeature()` 用于变更操作
- 变更成功后失效查询（重新获取列表）
- CRUD UI 使用 Ant Design Table + Form + Modal
- 页面通过 `../api` 访问 API，从不直接 `import from '@/api'`
