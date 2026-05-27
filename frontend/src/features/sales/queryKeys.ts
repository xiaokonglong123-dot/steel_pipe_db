import type { SalesOrderFilterParams } from './types';

export const salesQueryKeys = {
  all: ['sales-orders'] as const,
  list: (params?: SalesOrderFilterParams) => [...salesQueryKeys.all, params] as const,
  detail: (id: number) => ['sales-order', id] as const,
};
