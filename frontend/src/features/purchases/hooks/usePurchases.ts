import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { purchaseApi } from '../api/purchaseApi';
import { purchaseQueryKeys } from '../queryKeys';
import type {
  CreatePurchaseOrderData,
  PurchaseOrderFilterParams,
  PurchaseOrderStatusTransitionRequest,
} from '../types';

export function usePurchases(params?: PurchaseOrderFilterParams) {
  return useQuery({
    queryKey: purchaseQueryKeys.list(params),
    queryFn: () => purchaseApi.list(params),
  });
}

export function usePurchase(id: number) {
  return useQuery({
    queryKey: purchaseQueryKeys.detail(id),
    queryFn: () => purchaseApi.get(id),
    enabled: !!id,
  });
}

export function useCreatePurchaseOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreatePurchaseOrderData) => purchaseApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: purchaseQueryKeys.all });
    },
  });
}

export function useUpdatePurchaseOrder(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: Partial<CreatePurchaseOrderData>) => purchaseApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: purchaseQueryKeys.all });
      qc.invalidateQueries({ queryKey: purchaseQueryKeys.detail(id) });
    },
  });
}

export function useDeletePurchaseOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => purchaseApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: purchaseQueryKeys.all });
    },
  });
}

export function useTransitionPurchaseOrder(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: PurchaseOrderStatusTransitionRequest) => purchaseApi.transition(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: purchaseQueryKeys.all });
      qc.invalidateQueries({ queryKey: purchaseQueryKeys.detail(id) });
    },
  });
}

export function useApprovePurchaseOrder(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body?: { notes?: string }) => purchaseApi.approve(id, body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: purchaseQueryKeys.all });
      qc.invalidateQueries({ queryKey: purchaseQueryKeys.detail(id) });
    },
  });
}

export function useRejectPurchaseOrder(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: { reason: string }) => purchaseApi.reject(id, body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: purchaseQueryKeys.all });
      qc.invalidateQueries({ queryKey: purchaseQueryKeys.detail(id) });
    },
  });
}

export function useLinkInbound(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (inboundRecordId: number) => purchaseApi.linkInbound(id, inboundRecordId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: purchaseQueryKeys.all });
      qc.invalidateQueries({ queryKey: purchaseQueryKeys.detail(id) });
    },
  });
}
