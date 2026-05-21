import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type {
  SalesOrder,
  CreateSalesOrderData,
  SalesOrderFilterParams,
  SalesOrderStatusTransitionRequest,
  UpdateSalesOrderItemData,
} from '../types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { salesOrderSchema } from '@/zod-schemas/orders';

export const salesApi = {
  list: async (params?: SalesOrderFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<SalesOrder>>(
      '/sales-orders',
      { params },
    );
    return validateResponse(res.data.data, paginatedDataSchema(salesOrderSchema));
  },

  get: async (id: number) => {
    const res = await apiClient.get<ApiResponse<SalesOrder>>(`/sales-orders/${id}`);
    return validateResponse(res.data.data, salesOrderSchema);
  },

  create: async (data: CreateSalesOrderData) => {
    const res = await apiClient.post<ApiResponse<SalesOrder>>('/sales-orders', data);
    return validateResponse(res.data.data, salesOrderSchema);
  },

  update: async (id: number, data: Partial<CreateSalesOrderData>) => {
    const res = await apiClient.put<ApiResponse<SalesOrder>>(`/sales-orders/${id}`, data);
    return validateResponse(res.data.data, salesOrderSchema);
  },

  delete: async (id: number) => {
    await apiClient.delete(`/sales-orders/${id}`);
  },

  transition: async (id: number, data: SalesOrderStatusTransitionRequest) => {
    const res = await apiClient.post<ApiResponse<SalesOrder>>(`/sales-orders/${id}/transition`, data);
    return validateResponse(res.data.data, salesOrderSchema);
  },

  updateItem: async (orderId: number, itemId: number, data: UpdateSalesOrderItemData) => {
    const res = await apiClient.put<ApiResponse<SalesOrder>>(
      `/sales-orders/${orderId}/items/${itemId}`,
      data,
    );
    return validateResponse(res.data.data, salesOrderSchema);
  },

  deleteItem: async (orderId: number, itemId: number) => {
    await apiClient.delete(`/sales-orders/${orderId}/items/${itemId}`);
  },
};
