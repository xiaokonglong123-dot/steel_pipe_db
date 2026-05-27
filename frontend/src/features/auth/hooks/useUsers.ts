import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
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
  return useMutation({
    mutationFn: (data: CreateUserData) => userApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: userQueryKeys.all });
    },
  });
}

export function useUpdateUser() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: number; data: UpdateUserData }) =>
      userApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: userQueryKeys.all });
    },
  });
}

export function useChangePassword() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: number; data: ChangePasswordData }) =>
      userApi.changePassword(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: userQueryKeys.all });
    },
  });
}
