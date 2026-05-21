import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { purchaseApi } from '../api/purchaseApi';
import type {
  CreatePurchaseOrderData,
  PurchaseOrderFilterParams,
  PurchaseOrderStatusTransitionRequest,
} from '../types';

export function usePurchases(params?: PurchaseOrderFilterParams) {
  return useQuery({
    queryKey: ['purchase-orders', params],
    queryFn: () => purchaseApi.list(params),
  });
}

export function usePurchase(id: number) {
  return useQuery({
    queryKey: ['purchase-order', id],
    queryFn: () => purchaseApi.get(id),
    enabled: !!id,
  });
}

export function useCreatePurchaseOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreatePurchaseOrderData) => purchaseApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['purchase-orders'] });
    },
  });
}

export function useUpdatePurchaseOrder(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: Partial<CreatePurchaseOrderData>) => purchaseApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['purchase-orders'] });
      qc.invalidateQueries({ queryKey: ['purchase-order', id] });
    },
  });
}

export function useDeletePurchaseOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => purchaseApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['purchase-orders'] });
    },
  });
}

export function useTransitionPurchaseOrder(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: PurchaseOrderStatusTransitionRequest) => purchaseApi.transition(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['purchase-orders'] });
      qc.invalidateQueries({ queryKey: ['purchase-order', id] });
    },
  });
}
