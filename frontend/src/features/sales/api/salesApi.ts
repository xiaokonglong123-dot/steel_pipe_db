// 销售订单 API — CRUD + 状态流转 + 行项管理（含 ATP 库存校验）
import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type {
  SalesOrder,
  SalesOrderItem,
  CreateSalesOrderData,
  SalesOrderFilterParams,
  SalesOrderStatusTransitionRequest,
  UpdateSalesOrderItemData,
} from '../types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { salesOrderSchema, salesOrderDetailSchema } from '@/zod-schemas/orders';

export const salesApi = {
  list: async (params?: SalesOrderFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<SalesOrder>>(
      '/sales-orders',
      params as Record<string, unknown>,
    );
    return validateResponse(paginatedDataSchema(salesOrderSchema), res.data);
  },

  /** Get sales order detail — returns { order, items } structure. */
  get: async (id: number) => {
    const res = await apiClient.get<ApiResponse<{ order: SalesOrder; items: SalesOrderItem[] }>>(
      `/sales-orders/${id}`,
    );
    return validateResponse(salesOrderDetailSchema, res.data) as { order: SalesOrder; items: SalesOrderItem[] };
  },

  create: async (data: CreateSalesOrderData) => {
    const res = await apiClient.post<ApiResponse<SalesOrder>>('/sales-orders', data);
    return validateResponse(salesOrderSchema, res.data);
  },

  update: async (id: number, data: Partial<CreateSalesOrderData>) => {
    const res = await apiClient.put<ApiResponse<SalesOrder>>(`/sales-orders/${id}`, data);
    return validateResponse(salesOrderSchema, res.data);
  },

  delete: async (id: number) => {
    await apiClient.delete(`/sales-orders/${id}`);
  },

  approve: async (id: number, body?: { notes?: string }) => {
    const res = await apiClient.post<ApiResponse<SalesOrder>>(`/sales-orders/${id}/approve`, body ?? {});
    return validateResponse(salesOrderSchema, res.data);
  },

  reject: async (id: number, body: { reason: string }) => {
    const res = await apiClient.post<ApiResponse<SalesOrder>>(`/sales-orders/${id}/reject`, body);
    return validateResponse(salesOrderSchema, res.data);
  },

  linkOutbound: async (id: number, outboundRecordId: number) => {
    const res = await apiClient.post<ApiResponse<SalesOrder>>(`/sales-orders/${id}/link-outbound`, { outbound_record_id: outboundRecordId });
    return validateResponse(salesOrderSchema, res.data);
  },

  transition: async (id: number, data: SalesOrderStatusTransitionRequest) => {
    const res = await apiClient.post<ApiResponse<SalesOrder>>(`/sales-orders/${id}/transition`, data);
    return validateResponse(salesOrderSchema, res.data);
  },

  updateItem: async (orderId: number, itemId: number, data: UpdateSalesOrderItemData) => {
    const res = await apiClient.put<ApiResponse<SalesOrder>>(
      `/sales-orders/${orderId}/items/${itemId}`,
      data,
    );
    return validateResponse(salesOrderSchema, res.data);
  },

  deleteItem: async (orderId: number, itemId: number) => {
    await apiClient.delete(`/sales-orders/${orderId}/items/${itemId}`);
  },
};
