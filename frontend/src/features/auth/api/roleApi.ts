// RBAC API — roles, permissions, departments (auth domain)
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface Role {
  id: number;
  tenant_id: number;
  name: string;
  description: string | null;
  is_system: boolean;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
}

export interface Permission {
  id: number;
  key: string;
  name?: string;
  module?: string;
  description: string | null;
}

export interface Department {
  id: number;
  tenant_id: number;
  name: string;
  parent_id: number | null;
  path: string;
  sort_order: number;
  created_at: string;
}

// ---------------------------------------------------------------------------
// Schemas
// ---------------------------------------------------------------------------

const roleSchema = z.object({
  id: z.number(),
  tenant_id: z.number(),
  name: z.string(),
  description: z.string().nullable(),
  is_system: z.boolean(),
  created_at: z.string(),
  updated_at: z.string(),
  deleted_at: z.string().nullable(),
}).passthrough();

const permissionSchema = z.object({
  id: z.number(),
  key: z.string(),
  name: z.string().optional(),
  module: z.string().optional(),
  description: z.string().nullable(),
}).passthrough();

const departmentSchema = z.object({
  id: z.number(),
  tenant_id: z.number(),
  name: z.string(),
  parent_id: z.number().nullable(),
  path: z.string(),
  sort_order: z.number(),
  created_at: z.string(),
}).passthrough();

const arraySchema = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: z.array(item) }).passthrough();

// ---------------------------------------------------------------------------
// API
// ---------------------------------------------------------------------------

export const roleApi = {
  listRoles: async () => {
    const res = await apiClient.get<ApiResponse<Role[]>>('/auth/roles');
    return validateResponse(arraySchema(roleSchema), res.data).data;
  },

  createRole: async (data: { name: string; description?: string }) => {
    const res = await apiClient.post<ApiResponse<Role>>('/auth/roles', data);
    return validateResponse(roleSchema, res.data).data;
  },

  updateRole: async (id: number, data: { name?: string; description?: string }) => {
    const res = await apiClient.put<ApiResponse<Role>>(`/auth/roles/${id}`, data);
    return validateResponse(roleSchema, res.data).data;
  },

  deleteRole: async (id: number) => {
    await apiClient.delete(`/auth/roles/${id}`);
  },

  getRolePermissions: async (id: number) => {
    const res = await apiClient.get<ApiResponse<string[]>>(`/auth/roles/${id}/permissions`);
    return validateResponse(arraySchema(z.string()), res.data).data;
  },

  setRolePermissions: async (id: number, permissions: string[]) => {
    const res = await apiClient.put<ApiResponse<string[]>>(
      `/auth/roles/${id}/permissions`,
      { permissions },
    );
    return validateResponse(arraySchema(z.string()), res.data).data;
  },

  listPermissions: async () => {
    const res = await apiClient.get<ApiResponse<Permission[]>>('/auth/permissions');
    return validateResponse(arraySchema(permissionSchema), res.data).data;
  },

  listDepartments: async () => {
    const res = await apiClient.get<ApiResponse<Department[]>>('/auth/departments');
    return validateResponse(arraySchema(departmentSchema), res.data).data;
  },

  createDepartment: async (data: { name: string; parent_id?: number }) => {
    const res = await apiClient.post<ApiResponse<Department>>('/auth/departments', data);
    return validateResponse(departmentSchema, res.data).data;
  },

  deleteDepartment: async (id: number) => {
    await apiClient.delete(`/auth/departments/${id}`);
  },
};
