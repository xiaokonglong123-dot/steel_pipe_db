# `features/` — The Feature Module Pattern

Every feature module follows the same layout. Use this as a reference or template when adding a new one.

## Feature Module Structure

```
features/{feature}/
├── api/           ← TanStack Query hooks
│   └── index.ts   ← useQuery, useMutation
├── hooks/         ← Feature-specific React hooks (optional)
│   └── index.ts
├── queryKeys.ts   ← TanStack Query key factory for this feature
├── pages/         ← Page components (one per route)
│   ├── ListPage.tsx
│   ├── FormPage.tsx       ← Create + Edit combined
│   └── DetailPage.tsx
├── stores/        ← Zustand stores (optional)
│   └── index.ts
└── types/         ← TypeScript interfaces
    └── index.ts   ← Entity types, request/response types
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

- All API calls use the shared axios instance from `src/api/` (base URL: `/api/v1`).
- Query keys live in feature-local `queryKeys.ts` factories (for example `featureQueryKeys.all`, `.list(params)`, `.detail(id)`).
- Mutations invalidate the appropriate factory key on success to trigger refetch.
- Some features integrate `lib/validateResponse.ts` with Zod schemas from `zod-schemas/` for runtime response validation.

## Adding a New Feature Module

1. Create `features/{new_feature}/` with subdirs: `api/`, `hooks/`, `pages/`, `stores/`, `types/`.
2. Add `queryKeys.ts`, then write TanStack Query hooks in `api/index.ts` using that factory.
3. Build page components in `pages/`.
4. Add the route in `src/routes/index.tsx`.
5. Add i18n keys in `src/i18n/zh/{new_feature}.json` and `src/i18n/en/{new_feature}.json`.
6. If you need runtime API validation, add a Zod schema in `src/zod-schemas/`.

## Conventions

- `useFeatureQuery()` for lists, `useFeatureQuery(id)` for detail.
- `useCreateFeature()`, `useUpdateFeature()`, `useDeleteFeature()` for mutations.
- Always invalidate list queries after successful mutations.
- Do not add inline `queryKey: [...]` literals in feature API modules; centralize them in `queryKeys.ts`.
- CRUD UI uses Ant Design Table + Form + Modal.
- Pages import API through `../api`, never directly from `@/api`.
