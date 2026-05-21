// 认证 API — 登录、获取当前用户、刷新 Token
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
  // 登录：返回 JWT Token + 用户信息，由 authStore.setAuth 持久化
  login: async (data: LoginRequest) => {
    const res = await apiClient.post<ApiResponse<LoginResponse>>('/auth/login', data);
    return validateResponse(loginResponseSchema, res.data.data);
  },

  // 获取当前登录用户信息
  getMe: async () => {
    const res = await apiClient.get<ApiResponse<UserInfo>>('/auth/me');
    return validateResponse(userInfoSchema, res.data.data);
  },

  // 刷新 Token（当前未在前端主动调用）
  refreshToken: async (token: string) => {
    const res = await apiClient.post<ApiResponse<{ token: string }>>('/auth/refresh', { token });
    return validateResponse(tokenResponseSchema, res.data.data);
  },
};
