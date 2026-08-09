import type { UserFilterParams } from './api/userApi';

export const userQueryKeys = {
  all: ['users'] as const,
  list: (params?: UserFilterParams) => [...userQueryKeys.all, params] as const,
};

export const roleQueryKeys = {
  roles: ['roles'] as const,
  permissions: ['permissions'] as const,
  rolePermissions: (roleId?: number) => ['role-permissions', roleId] as const,
};

export const departmentQueryKeys = {
  all: ['departments'] as const,
};
