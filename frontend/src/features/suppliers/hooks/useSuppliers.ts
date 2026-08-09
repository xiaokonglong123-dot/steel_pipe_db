import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { message } from 'antd';
import { useTranslation } from 'react-i18next';
import { supplierApi } from '../api/supplierApi';
import { supplierQueryKeys } from '../queryKeys';
import type { Supplier, CreateSupplierData, SupplierFilterParams } from '../types';
import type { PaginatedData } from '@/types';

export function useSuppliers(params?: SupplierFilterParams) {
  return useQuery<PaginatedData<Supplier>>({
    queryKey: supplierQueryKeys.list(params),
    queryFn: () => supplierApi.list(params),
  });
}

export function useSupplier(id: number) {
  return useQuery<Supplier>({
    queryKey: supplierQueryKeys.detail(id),
    queryFn: () => supplierApi.getById(id),
    enabled: !!id,
  });
}

export function useCreateSupplier() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: CreateSupplierData) => supplierApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: supplierQueryKeys.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useUpdateSupplier(id: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: Partial<CreateSupplierData>) => supplierApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: supplierQueryKeys.all });
      qc.invalidateQueries({ queryKey: supplierQueryKeys.detail(id) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useDeleteSupplier() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (id: number) => supplierApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: supplierQueryKeys.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useSupplierSearch(q: string) {
  return useQuery({
    queryKey: supplierQueryKeys.search(q),
    queryFn: () => supplierApi.search(q),
    enabled: q.length > 0,
  });
}

export function useActiveSuppliers() {
  return useQuery({
    queryKey: supplierQueryKeys.active(),
    queryFn: () => supplierApi.listActive(),
  });
}
