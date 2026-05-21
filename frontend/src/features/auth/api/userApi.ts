import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse, UserInfo } from '@/types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { z } from 'zod';
import { userInfoSchema } from '@/zod-schemas/core';

export interface CreateUserData {
  username: string;
  password: string;
  display_name: string;
  role: string;
  email?: string;
  phone?: string;
}

export interface UpdateUserData {
  display_name?: string;
  role?: string;
  email?: string;
  phone?: string;
  is_active?: boolean;
}

export interface ChangePasswordData {
  new_password: string;
}

export interface UserFilterParams {
  page?: number;
  page_size?: number;
  q?: string;
}

export const userApi = {
  list: async (params?: UserFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<UserInfo>>('/users', { params });
    return validateResponse(res.data.data, paginatedDataSchema(userInfoSchema));
  },

  create: async (data: CreateUserData) => {
    const res = await apiClient.post<ApiResponse<UserInfo>>('/users', data);
    return validateResponse(res.data.data, userInfoSchema);
  },

  update: async (id: number, data: UpdateUserData) => {
    const res = await apiClient.put<ApiResponse<UserInfo>>(`/users/${id}`, data);
    return validateResponse(res.data.data, userInfoSchema);
  },

  changePassword: async (id: number, data: ChangePasswordData) => {
    const res = await apiClient.post<ApiResponse<null>>(`/users/${id}/change-password`, data);
    return validateResponse(res.data, z.object({ success: z.boolean(), data: z.null() }));
  },
};
