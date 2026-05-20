import client from './client';

export interface LoginDto {
  username: string;
  password: string;
}

export interface UserInfo {
  id: string;
  username: string;
  display_name: string;
  role: string;
  email?: string;
  phone?: string;
  is_active: boolean;
}

export interface LoginResponse {
  access_token: string;
  refresh_token: string;
  user: UserInfo;
}

export const authApi = {
  login: (data: LoginDto) =>
    client.post<{ success: boolean; data: LoginResponse }>('/auth/login', data),

  refresh: (refreshToken: string) =>
    client.post<{ success: boolean; data: { access_token: string; refresh_token: string } }>(
      '/auth/refresh',
      { refresh_token: refreshToken }
    ),

  me: () =>
    client.get<{ success: boolean; data: UserInfo }>('/auth/me'),

  listUsers: (params?: { search?: string; role?: string }) =>
    client.get<{ success: boolean; data: UserInfo[] }>('/auth/users', { params }),

  createUser: (data: { username: string; password: string; display_name: string; role: string; email?: string; phone?: string }) =>
    client.post<{ success: boolean; data: UserInfo }>('/auth/users', data),

  updateUser: (id: string, data: Partial<UserInfo>) =>
    client.put<{ success: boolean; data: UserInfo }>(`/auth/users/${id}`, data),

  deleteUser: (id: string) =>
    client.delete<{ success: boolean }>(`/auth/users/${id}`),
};
