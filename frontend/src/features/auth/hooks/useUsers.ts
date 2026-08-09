import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { message } from 'antd';
import { useTranslation } from 'react-i18next';
import { userApi } from '../api/userApi';
import { userQueryKeys } from '../queryKeys';
import type { CreateUserData, UpdateUserData, ChangePasswordData, UserFilterParams } from '../api/userApi';

export function useUsers(params?: UserFilterParams) {
  return useQuery({
    queryKey: userQueryKeys.list(params),
    queryFn: () => userApi.list(params),
  });
}

export function useCreateUser() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: CreateUserData) => userApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: userQueryKeys.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useUpdateUser() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: ({ id, data }: { id: number; data: UpdateUserData }) =>
      userApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: userQueryKeys.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useChangePassword() {
  const { t } = useTranslation();
  return useMutation({
    mutationFn: ({ id, data }: { id: number; data: ChangePasswordData }) =>
      userApi.changePassword(id, data),
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}
