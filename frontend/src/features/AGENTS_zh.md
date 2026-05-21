# `features/` — 特性模块模式

所有 6 个特性模块遵循相同的模式。本文档作为理解或添加任何特性的模板。

## 特性模块结构

```
features/{特性}/
├── api/           ← TanStack Query hooks（API 层）
│   └── index.ts   ← useQuery、useMutation hooks
├── hooks/         ← 特性专属 React hooks（可选）
│   └── index.ts
├── pages/         ← 页面组件（1 个文件 = 1 条路由）
│   ├── ListPage.tsx
│   ├── CreatePage.tsx
│   ├── EditPage.tsx
│   └── DetailPage.tsx
└── types/         ← TypeScript 接口
    └── index.ts   ← 实体类型、请求/响应类型
```

## 现有特性

| 特性 | 路由 | 描述 |
|---------|--------|-------------|
| `pipes/` | `/pipes/*` | 钢管规格管理（CRUD + 列表）|
| `inventory/` | `/inventory/*` | 库存追踪 |
| `purchases/` | `/purchases/*` | 采购订单管理 |
| `reports/` | `/reports/*` | 查看、过滤、导出报表 |
| `production/` | `/production/*` | 生产订单管理 |
| `customers/` | `/customers/*` | 客户关系管理 |

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

## 添加新特性模块
1. 创建 `features/{new_feature}/`，包含 `api/`、`hooks/`、`pages/`、`types/` 子目录
2. 在 `api/index.ts` 中添加 TanStack Query hooks
3. 在 `pages/` 中添加页面组件
4. 在 `src/routes/index.tsx` 中添加路由
5. 在 `src/i18n/zh/{new_feature}.json` 和 `src/i18n/en/{new_feature}.json` 中添加 i18n 键

## 约定
- `useFeatureQuery()` 用于列表，`useFeatureQuery(id)` 用于详情
- `useCreateFeature()`、`useUpdateFeature()`、`useDeleteFeature()` 用于变更操作
- 变更成功后失效查询（重新获取列表）
- CRUD UI 使用 Ant Design Table + Form + Modal
- 页面通过 `../api` 访问 API，从不直接 `import from '@/api'`
