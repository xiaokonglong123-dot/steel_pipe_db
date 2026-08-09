import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { message } from 'antd';
import { useTranslation } from 'react-i18next';
import { contractApi } from '../api/contractApi';
import { contractQueryKeys } from '../queryKeys';
import type {
  CreateContractData,
  CreateContractItemData,
  CreateContractPaymentData,
  ContractFilterParams,
} from '../types';

export function useContracts(params?: ContractFilterParams) {
  return useQuery({
    queryKey: contractQueryKeys.list(params),
    queryFn: () => contractApi.list(params),
  });
}

export function useContract(id: number) {
  return useQuery({
    queryKey: contractQueryKeys.detail(id),
    queryFn: () => contractApi.get(id),
  });
}

export function useContractDetail(id: number) {
  return useQuery({
    queryKey: contractQueryKeys.detail(id),
    queryFn: () => contractApi.getDetail(id),
    enabled: !!id,
  });
}

export function useCreateContract() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: CreateContractData) => contractApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: contractQueryKeys.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useUpdateContract(id: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: Partial<CreateContractData>) => contractApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: contractQueryKeys.all });
      qc.invalidateQueries({ queryKey: contractQueryKeys.detail(id) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useDeleteContract() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (id: number) => contractApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: contractQueryKeys.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useUpdateContractStatus(id: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (status: string) => contractApi.updateStatus(id, status),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: contractQueryKeys.all });
      qc.invalidateQueries({ queryKey: contractQueryKeys.detail(id) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useCreateContractItem(contractId: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: CreateContractItemData) => contractApi.addItem(contractId, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: contractQueryKeys.detail(contractId) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useUpdateContractItem(contractId: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: ({ id, data }: { id: number; data: Partial<CreateContractItemData> }) =>
      contractApi.updateItem(contractId, id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: contractQueryKeys.detail(contractId) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useDeleteContractItem(contractId: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (itemId: number) => contractApi.deleteItem(contractId, itemId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: contractQueryKeys.detail(contractId) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useContractPayments(contractId: number) {
  return useQuery({
    queryKey: contractQueryKeys.payments(contractId),
    queryFn: () => contractApi.listPayments(contractId),
    enabled: !!contractId,
  });
}

export function useCreateContractPayment(contractId: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: CreateContractPaymentData) => contractApi.addPayment(contractId, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: contractQueryKeys.detail(contractId) });
      qc.invalidateQueries({ queryKey: contractQueryKeys.payments(contractId) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useUpdateContractPayment(contractId: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: ({ id, data }: { id: number; data: Partial<CreateContractPaymentData> }) =>
      contractApi.updatePayment(contractId, id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: contractQueryKeys.detail(contractId) });
      qc.invalidateQueries({ queryKey: contractQueryKeys.payments(contractId) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useDeleteContractPayment(contractId: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (paymentId: number) => contractApi.deletePayment(contractId, paymentId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: contractQueryKeys.detail(contractId) });
      qc.invalidateQueries({ queryKey: contractQueryKeys.payments(contractId) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}
