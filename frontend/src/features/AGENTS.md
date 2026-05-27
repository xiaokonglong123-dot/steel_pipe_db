# `features/` — The Feature Module Pattern

All 13 feature modules follow the same layout. This doc is both a reference and a template for adding new ones.

## Feature Module Structure

```
features/{feature}/
├── api/           ← TanStack Query hooks
│   └── index.ts   ← useQuery, useMutation
├── hooks/         ← Feature-specific React hooks (optional)
│   └── index.ts
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
|---------|--------|-------------|
| `auth/` | (via ProtectedRoute) | Login, auth state via Zustand |
| `pipes/` | `/pipes/seamless/*`, `/pipes/screen/*` | API 5CT pipe master data (seamless + screen) |
| `inventory/` | `/inventory/inbound`, `/inventory/outbound`, `/inventory/stock`, `/inventory/locations`, `/inventory/check` | Stock tracking, in/out, locations, checks |
| `suppliers/` | `/suppliers/*` | Supplier management |
| `customers/` | `/customers/*` | Customer management |
| `purchases/` | `/purchases/*` | Purchase orders, approval workflow |
| `sales/` | `/sales/*` | Sales orders, ATP check |
| `quality/` | `/quality/certs/*` | Quality certs, mechanical/NDT tests |
| `contracts/` | `/contracts/*` | Sales/procurement contracts, payment milestones |
| `reports/` | `/reports`, `/reports/dashboard` | Dashboard, daily/monthly/statistical reports |
| `labels/` | `/labels` | Barcode and spec label generation |
| `search/` | `/search` | Global search across pipes, inventory, orders |
| `profile/` | `/profile/settings` | User settings, password change |

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

- All API calls go through the shared axios instance at `src/api/` (base URL: `/api/v1`).
- Query key convention: `['entity']` for list, `['entity', id]` for detail.
- Mutations invalidate the list query on success so it refetches.
- Some features use `lib/validateResponse.ts` with Zod schemas from `zod-schemas/` for runtime response validation.

## Adding a New Feature Module

1. Create `features/{new_feature}/` with subdirs: `api/`, `hooks/`, `pages/`, `stores/`, `types/`.
2. Write TanStack Query hooks in `api/index.ts`.
3. Build page components in `pages/`.
4. Register the route in `src/routes/index.tsx`.
5. Add i18n keys in both `src/i18n/zh/{new_feature}.json` and `src/i18n/en/{new_feature}.json`.
6. Add a Zod response schema in `src/zod-schemas/` if you want runtime validation.

## Conventions

- `useFeatureQuery()` for list queries, `useFeatureQuery(id)` for detail.
- `useCreateFeature()`, `useUpdateFeature()`, `useDeleteFeature()` for mutations.
- Always invalidate list queries after successful mutations.
- CRUD UI uses Ant Design Table + Form + Modal.
- Pages import API through `../api`, never directly from `@/api`.
