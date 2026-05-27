// Auth API — login, get current user, refresh token
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
  // Login: returns JWT + user info, persisted by authStore.setAuth
  login: async (data: LoginRequest) => {
    const res = await apiClient.post<ApiResponse<LoginResponse>>('/auth/login', data);
    return validateResponse(loginResponseSchema, res.data.data);
  },

  // Get current logged-in user info
  getMe: async () => {
    const res = await apiClient.get<ApiResponse<UserInfo>>('/auth/me');
    return validateResponse(userInfoSchema, res.data.data);
  },

  // Refresh token (not actively called from frontend rn)
  refreshToken: async (token: string) => {
    const res = await apiClient.post<ApiResponse<{ token: string }>>('/auth/refresh', { token });
    return validateResponse(tokenResponseSchema, res.data.data);
  },
};
