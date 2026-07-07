import type { UserFilterParams } from './api/userApi';

export const userQueryKeys = {
  all: ['users'] as const,
  list: (params?: UserFilterParams) => [...userQueryKeys.all, params] as const,
};
