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
    return validateResponse(res.data.data, loginResponseSchema);
  },

  getMe: async () => {
    const res = await apiClient.get<ApiResponse<UserInfo>>('/auth/me');
    return validateResponse(res.data.data, userInfoSchema);
  },

  refreshToken: async (token: string) => {
    const res = await apiClient.post<ApiResponse<{ token: string }>>('/auth/refresh', { token });
    return validateResponse(res.data.data, tokenResponseSchema);
  },
};
