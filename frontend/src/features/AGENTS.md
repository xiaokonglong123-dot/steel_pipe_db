# `features/` — Feature Module Pattern

All 13 feature modules follow an identical pattern. This document serves as the template for understanding or adding any feature.

## Feature Module Structure

```
features/{feature}/
├── api/           ← TanStack Query hooks (API layer)
│   └── index.ts   ← useQuery, useMutation hooks
├── hooks/         ← Feature-specific React hooks (optional)
│   └── index.ts
├── pages/         ← Page components (1 file = 1 route)
│   ├── ListPage.tsx
│   ├── FormPage.tsx       ← Create + Edit combined
│   └── DetailPage.tsx
├── stores/        ← Zustand stores (optional, for complex features)
│   └── index.ts
└── types/         ← TypeScript interfaces
    └── index.ts   ← Entity types, request/response types
```

## Existing Features

| Feature | Routes | Description |
|---------|--------|-------------|
| `auth/` | (via ProtectedRoute) | Login page, auth state management (Zustand) |
| `pipes/` | `/pipes/seamless/*`, `/pipes/screen/*` | API 5CT pipe master data (seamless + screen) |
| `inventory/` | `/inventory/inbound`, `/inventory/outbound`, `/inventory/stock`, `/inventory/locations`, `/inventory/check` | Stock tracking, inbound/outbound, location management, inventory checks |
| `suppliers/` | `/suppliers/*` | Supplier management |
| `customers/` | `/customers/*` | Customer management |
| `purchases/` | `/purchases/*` | Purchase orders, approval workflow |
| `sales/` | `/sales/*` | Sales orders, ATP check |
| `quality/` | `/quality/certs/*` | Quality certificates, mechanical/NDT tests |
| `contracts/` | `/contracts/*` | Sales/procurement contracts, payment milestones |
| `reports/` | `/reports`, `/reports/dashboard` | Dashboard, daily/monthly/statistical reports |
| `labels/` | `/labels` | Barcode and specification label generation |
| `search/` | `/search` | Global search across pipes, inventory, orders |
| `profile/` | `/profile/settings` | User profile settings, password change |

## Template: `api/index.ts` (TanStack Query Hooks)
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
- All API calls use the shared axios instance from `src/api/` (base URL: `/api/v1`)
- Query keys follow convention: `['entity']` for list, `['entity', id]` for detail
- Mutations invalidate the list query key on success
- Some feature API modules integrate `lib/validateResponse.ts` for runtime response validation via Zod schemas in `zod-schemas/`

## Adding a New Feature Module
1. Create `features/{new_feature}/` with `api/`, `hooks/`, `pages/`, `stores/`, `types/` subdirs
2. Add TanStack Query hooks in `api/index.ts`
3. Add page components in `pages/`
4. Add route in `src/routes/index.tsx`
5. Add i18n keys in `src/i18n/zh/{new_feature}.json` and `src/i18n/en/{new_feature}.json`
6. Add Zod response schema in `src/zod-schemas/` if API validation is needed

## Conventions
- `useFeatureQuery()` for list, `useFeatureQuery(id)` for detail
- `useCreateFeature()`, `useUpdateFeature()`, `useDeleteFeature()` for mutations
- Invalid queries on mutation success (refetch list)
- Ant Design Table + Form + Modal for CRUD UI
- Pages access the API through `../api`, never directly importing from `@/api`
