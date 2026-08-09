import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { message } from 'antd';
import { useTranslation } from 'react-i18next';
import { salesApi } from '../api/salesApi';
import { salesQueryKeys } from '../queryKeys';
import type {
  CreateSalesOrderData,
  SalesOrderFilterParams,
  SalesOrderStatusTransitionRequest,
  UpdateSalesOrderItemData,
} from '../types';

export function useSalesOrders(params?: SalesOrderFilterParams) {
  return useQuery({
    queryKey: salesQueryKeys.list(params),
    queryFn: () => salesApi.list(params),
  });
}

export function useSalesOrder(id: number) {
  return useQuery({
    queryKey: salesQueryKeys.detail(id),
    queryFn: () => salesApi.get(id),
    enabled: !!id,
  });
}

export function useCreateSalesOrder() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: CreateSalesOrderData) => salesApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: salesQueryKeys.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useUpdateSalesOrder(id: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: Partial<CreateSalesOrderData>) => salesApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: salesQueryKeys.all });
      qc.invalidateQueries({ queryKey: salesQueryKeys.detail(id) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useDeleteSalesOrder() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (id: number) => salesApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: salesQueryKeys.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useTransitionSalesOrder(id: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: SalesOrderStatusTransitionRequest) => salesApi.transition(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: salesQueryKeys.all });
      qc.invalidateQueries({ queryKey: salesQueryKeys.detail(id) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useApproveSalesOrder(id: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (body?: { notes?: string }) => salesApi.approve(id, body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: salesQueryKeys.all });
      qc.invalidateQueries({ queryKey: salesQueryKeys.detail(id) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useRejectSalesOrder(id: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (body: { reason: string }) => salesApi.reject(id, body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: salesQueryKeys.all });
      qc.invalidateQueries({ queryKey: salesQueryKeys.detail(id) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useLinkOutbound(id: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (outboundRecordId: number) => salesApi.linkOutbound(id, outboundRecordId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: salesQueryKeys.all });
      qc.invalidateQueries({ queryKey: salesQueryKeys.detail(id) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useUpdateSalesOrderItem(orderId: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: ({ itemId, data }: { itemId: number; data: UpdateSalesOrderItemData }) =>
      salesApi.updateItem(orderId, itemId, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: salesQueryKeys.detail(orderId) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useDeleteSalesOrderItem(orderId: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (itemId: number) => salesApi.deleteItem(orderId, itemId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: salesQueryKeys.detail(orderId) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}
