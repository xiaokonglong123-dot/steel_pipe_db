import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type {
  SalesOrder,
  CreateSalesOrderData,
  SalesOrderFilterParams,
  SalesOrderStatusTransitionRequest,
  UpdateSalesOrderItemData,
} from '../types';

export const salesApi = {
  list: async (params?: SalesOrderFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<SalesOrder>>(
      '/sales-orders',
      { params },
    );
    return res.data.data;
  },

  get: async (id: number) => {
    const res = await apiClient.get<ApiResponse<SalesOrder>>(`/sales-orders/${id}`);
    return res.data.data;
  },

  create: async (data: CreateSalesOrderData) => {
    const res = await apiClient.post<ApiResponse<SalesOrder>>('/sales-orders', data);
    return res.data.data;
  },

  update: async (id: number, data: Partial<CreateSalesOrderData>) => {
    const res = await apiClient.put<ApiResponse<SalesOrder>>(`/sales-orders/${id}`, data);
    return res.data.data;
  },

  delete: async (id: number) => {
    await apiClient.delete(`/sales-orders/${id}`);
  },

  transition: async (id: number, data: SalesOrderStatusTransitionRequest) => {
    const res = await apiClient.post<ApiResponse<SalesOrder>>(`/sales-orders/${id}/transition`, data);
    return res.data.data;
  },

  updateItem: async (orderId: number, itemId: number, data: UpdateSalesOrderItemData) => {
    const res = await apiClient.put<ApiResponse<SalesOrder>>(
      `/sales-orders/${orderId}/items/${itemId}`,
      data,
    );
    return res.data.data;
  },

  deleteItem: async (orderId: number, itemId: number) => {
    await apiClient.delete(`/sales-orders/${orderId}/items/${itemId}`);
  },
};
