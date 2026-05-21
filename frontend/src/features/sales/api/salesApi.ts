// 销售订单 API — CRUD + 状态流转 + 行项管理（含 ATP 库存校验）
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
    return validateResponse(paginatedDataSchema(salesOrderSchema), res.data.data);
  },

  get: async (id: number) => {
    const res = await apiClient.get<ApiResponse<SalesOrder>>(`/sales-orders/${id}`);
    return validateResponse(salesOrderSchema, res.data.data);
  },

  create: async (data: CreateSalesOrderData) => {
    const res = await apiClient.post<ApiResponse<SalesOrder>>('/sales-orders', data);
    return validateResponse(salesOrderSchema, res.data.data);
  },

  update: async (id: number, data: Partial<CreateSalesOrderData>) => {
    const res = await apiClient.put<ApiResponse<SalesOrder>>(`/sales-orders/${id}`, data);
    return validateResponse(salesOrderSchema, res.data.data);
  },

  delete: async (id: number) => {
    await apiClient.delete(`/sales-orders/${id}`);
  },

  // 状态流转：pending → approved → delivered → invoiced
  transition: async (id: number, data: SalesOrderStatusTransitionRequest) => {
    const res = await apiClient.post<ApiResponse<SalesOrder>>(`/sales-orders/${id}/transition`, data);
    return validateResponse(salesOrderSchema, res.data.data);
  },

  updateItem: async (orderId: number, itemId: number, data: UpdateSalesOrderItemData) => {
    const res = await apiClient.put<ApiResponse<SalesOrder>>(
      `/sales-orders/${orderId}/items/${itemId}`,
      data,
    );
    return validateResponse(salesOrderSchema, res.data.data);
  },

  deleteItem: async (orderId: number, itemId: number) => {
    await apiClient.delete(`/sales-orders/${orderId}/items/${itemId}`);
  },
};
