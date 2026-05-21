import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type {
  PurchaseOrder,
  CreatePurchaseOrderData,
  PurchaseOrderFilterParams,
  PurchaseOrderStatusTransitionRequest,
} from '../types';

export const purchaseApi = {
  list: async (params?: PurchaseOrderFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<PurchaseOrder>>(
      '/purchase-orders',
      { params },
    );
    return res.data.data;
  },

  get: async (id: number) => {
    const res = await apiClient.get<ApiResponse<PurchaseOrder>>(`/purchase-orders/${id}`);
    return res.data.data;
  },

  create: async (data: CreatePurchaseOrderData) => {
    const res = await apiClient.post<ApiResponse<PurchaseOrder>>('/purchase-orders', data);
    return res.data.data;
  },

  update: async (id: number, data: Partial<CreatePurchaseOrderData>) => {
    const res = await apiClient.put<ApiResponse<PurchaseOrder>>(`/purchase-orders/${id}`, data);
    return res.data.data;
  },

  delete: async (id: number) => {
    await apiClient.delete(`/purchase-orders/${id}`);
  },

  transition: async (id: number, data: PurchaseOrderStatusTransitionRequest) => {
    const res = await apiClient.post<ApiResponse<PurchaseOrder>>(
      `/purchase-orders/${id}/transition`,
      data,
    );
    return res.data.data;
  },

  updateItem: async (orderId: number, itemId: number, data: Record<string, unknown>) => {
    const res = await apiClient.put<ApiResponse<PurchaseOrder>>(
      `/purchase-orders/${orderId}/items/${itemId}`,
      data,
    );
    return res.data.data;
  },

  deleteItem: async (orderId: number, itemId: number) => {
    await apiClient.delete(`/purchase-orders/${orderId}/items/${itemId}`);
  },
};
