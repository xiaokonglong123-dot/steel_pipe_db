import apiClient from '@/api/client';
import type { ApiResponse, UserInfo } from '@/types';

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
    return res.data.data;
  },

  getMe: async () => {
    const res = await apiClient.get<ApiResponse<UserInfo>>('/auth/me');
    return res.data.data;
  },

  refreshToken: async (token: string) => {
    const res = await apiClient.post<ApiResponse<{ token: string }>>('/auth/refresh', { token });
    return res.data.data;
  },
};
