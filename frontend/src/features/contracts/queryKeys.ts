import type { ContractFilterParams } from './types';

export const contractQueryKeys = {
  all: ['contracts'] as const,
  list: (params?: ContractFilterParams) => [...contractQueryKeys.all, params] as const,
  detail: (id: number) => ['contract', id] as const,
  payments: (contractId: number) => ['contract-payments', contractId] as const,
};
