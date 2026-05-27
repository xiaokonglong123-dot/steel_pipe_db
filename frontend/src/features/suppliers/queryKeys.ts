import type { SupplierFilterParams } from './types';

export const supplierQueryKeys = {
  all: ['suppliers'] as const,
  list: (params?: SupplierFilterParams) => [...supplierQueryKeys.all, params] as const,
  detail: (id: number) => ['supplier', id] as const,
  search: (q: string) => [...supplierQueryKeys.all, 'search', q] as const,
  active: () => [...supplierQueryKeys.all, 'active'] as const,
};
