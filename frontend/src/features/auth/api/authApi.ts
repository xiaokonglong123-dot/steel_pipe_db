import apiClient from '@/api/client';
import type { ApiResponse, UserInfo } from '@/types';
import { validateResponse } from '@/lib/validateResponse';
import { loginResponseSchema, userInfoSchema, tokenResponseSchema } from '@/zod-schemas/core';

export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  user: UserInfo;
}

export const authApi = {
  login: async (data: LoginRequest) => {
    const res = await apiClient.post<ApiResponse<LoginResponse>>('/auth/login', data);
    return validateResponse(loginResponseSchema, res.data);
  },

  logout: async () => {
    await apiClient.post('/auth/logout');
  },

  getMe: async () => {
    const res = await apiClient.get<ApiResponse<UserInfo>>('/auth/me');
    return validateResponse(userInfoSchema, res.data);
  },

  refresh: async () => {
    const res = await apiClient.post<ApiResponse<{ token: string; refresh_token: string }>>('/auth/refresh');
    return validateResponse(tokenResponseSchema, res.data);
  },
};
