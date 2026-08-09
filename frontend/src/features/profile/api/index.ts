import { useMutation, useQueryClient } from '@tanstack/react-query';
import { message } from 'antd';
import { useTranslation } from 'react-i18next';
import apiClient from '@/api/client';
import { useAuthStore } from '@/stores/authStore';
import { userQueryKeys } from '@/features/auth/queryKeys';
import type { ApiResponse } from '@/types';
import type { UserInfo } from '@/types';

export interface UpdateProfileData {
  display_name: string;
  email?: string;
  phone?: string;
}

export interface ChangePasswordData {
  old_password: string;
  new_password: string;
}

export function useUpdateProfile() {
  const qc = useQueryClient();
  const setUser = useAuthStore((s) => s.setUser);
  const { t } = useTranslation();

  return useMutation({
    mutationFn: (data: UpdateProfileData) =>
      apiClient.put<{ success: boolean; data: UserInfo }>('/auth/me', {
        display_name: data.display_name,
        email: data.email,
        phone: data.phone,
      }),
    onSuccess: (res) => {
      if (res.success && res.data) {
        setUser(res.data);
      }
      qc.invalidateQueries({ queryKey: userQueryKeys.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useChangePassword() {
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: ChangePasswordData) => {
      const userId = useAuthStore.getState().user?.id;
      return apiClient.post<ApiResponse<string>>(`/users/${userId}/change-password`, data);
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}
