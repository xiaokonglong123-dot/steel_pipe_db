import type { CustomerFilterParams } from './types';

export const customerQueryKeys = {
  all: ['customers'] as const,
  list: (params?: CustomerFilterParams) => [...customerQueryKeys.all, params] as const,
  detail: (id: number) => ['customer', id] as const,
  search: (q: string) => [...customerQueryKeys.all, 'search', q] as const,
  active: () => [...customerQueryKeys.all, 'active'] as const,
};
