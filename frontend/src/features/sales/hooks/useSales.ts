import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { salesApi } from '../api/salesApi';
import type {
  CreateSalesOrderData,
  SalesOrderFilterParams,
  SalesOrderStatusTransitionRequest,
  UpdateSalesOrderItemData,
} from '../types';

export function useSalesOrders(params?: SalesOrderFilterParams) {
  return useQuery({
    queryKey: ['sales-orders', params],
    queryFn: () => salesApi.list(params),
  });
}

export function useSalesOrder(id: number) {
  return useQuery({
    queryKey: ['sales-order', id],
    queryFn: () => salesApi.get(id),
    enabled: !!id,
  });
}

export function useCreateSalesOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateSalesOrderData) => salesApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['sales-orders'] });
    },
  });
}

export function useUpdateSalesOrder(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: Partial<CreateSalesOrderData>) => salesApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['sales-orders'] });
      qc.invalidateQueries({ queryKey: ['sales-order', id] });
    },
  });
}

export function useDeleteSalesOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => salesApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['sales-orders'] });
    },
  });
}

export function useTransitionSalesOrder(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: SalesOrderStatusTransitionRequest) => salesApi.transition(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['sales-orders'] });
      qc.invalidateQueries({ queryKey: ['sales-order', id] });
    },
  });
}

export function useUpdateSalesOrderItem(orderId: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ itemId, data }: { itemId: number; data: UpdateSalesOrderItemData }) =>
      salesApi.updateItem(orderId, itemId, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['sales-order', orderId] });
    },
  });
}

export function useDeleteSalesOrderItem(orderId: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (itemId: number) => salesApi.deleteItem(orderId, itemId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['sales-order', orderId] });
    },
  });
}
