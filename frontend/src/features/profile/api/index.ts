// 个人设置 API — 更新资料 + 修改密码
import { useMutation, useQueryClient } from '@tanstack/react-query';
import apiClient from '@/api/client';
import { useAuthStore } from '@/stores/authStore';
import type { UserInfo } from '@/types';

export interface UpdateProfileData {
  id: number;
  display_name: string;
  email?: string;
  phone?: string;
}

export interface ChangePasswordData {
  current_password: string;
  new_password: string;
}

export function useUpdateProfile() {
  const qc = useQueryClient();
  const setUser = useAuthStore((s) => s.setUser);

  return useMutation({
    mutationFn: (data: UpdateProfileData) =>
      apiClient.put<{ success: boolean; data: UserInfo }>(`/users/${data.id}`, {
        display_name: data.display_name,
        email: data.email,
        phone: data.phone,
      }),
    onSuccess: (res) => {
      if (res.data.success && res.data.data) {
        setUser(res.data.data);
      }
      qc.invalidateQueries({ queryKey: ['users'] });
    },
  });
}

export function useChangePassword() {
  return useMutation({
    mutationFn: (data: ChangePasswordData) =>
      apiClient.put('/auth/me/change-password', data),
  });
}
