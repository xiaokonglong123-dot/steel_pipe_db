import type { PurchaseOrderFilterParams } from './types';

export const purchaseQueryKeys = {
  all: ['purchase-orders'] as const,
  list: (params?: PurchaseOrderFilterParams) => [...purchaseQueryKeys.all, params] as const,
  detail: (id: number) => ['purchase-order', id] as const,
};
