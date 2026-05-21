import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type {
  PurchaseOrder,
  CreatePurchaseOrderData,
  PurchaseOrderFilterParams,
  PurchaseOrderStatusTransitionRequest,
} from '../types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { purchaseOrderSchema } from '@/zod-schemas/orders';

export const purchaseApi = {
  list: async (params?: PurchaseOrderFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<PurchaseOrder>>(
      '/purchase-orders',
      { params },
    );
    return validateResponse(res.data.data, paginatedDataSchema(purchaseOrderSchema));
  },

  get: async (id: number) => {
    const res = await apiClient.get<ApiResponse<PurchaseOrder>>(`/purchase-orders/${id}`);
    return validateResponse(res.data.data, purchaseOrderSchema);
  },

  create: async (data: CreatePurchaseOrderData) => {
    const res = await apiClient.post<ApiResponse<PurchaseOrder>>('/purchase-orders', data);
    return validateResponse(res.data.data, purchaseOrderSchema);
  },

  update: async (id: number, data: Partial<CreatePurchaseOrderData>) => {
    const res = await apiClient.put<ApiResponse<PurchaseOrder>>(`/purchase-orders/${id}`, data);
    return validateResponse(res.data.data, purchaseOrderSchema);
  },

  delete: async (id: number) => {
    await apiClient.delete(`/purchase-orders/${id}`);
  },

  transition: async (id: number, data: PurchaseOrderStatusTransitionRequest) => {
    const res = await apiClient.post<ApiResponse<PurchaseOrder>>(
      `/purchase-orders/${id}/transition`,
      data,
    );
    return validateResponse(res.data.data, purchaseOrderSchema);
  },

  updateItem: async (orderId: number, itemId: number, data: Record<string, unknown>) => {
    const res = await apiClient.put<ApiResponse<PurchaseOrder>>(
      `/purchase-orders/${orderId}/items/${itemId}`,
      data,
    );
    return validateResponse(res.data.data, purchaseOrderSchema);
  },

  deleteItem: async (orderId: number, itemId: number) => {
    await apiClient.delete(`/purchase-orders/${orderId}/items/${itemId}`);
  },
};
