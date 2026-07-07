// 采购订单 API — CRUD + 状态流转 + 行项管理
import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type {
  PurchaseOrder,
  PurchaseOrderItem,
  CreatePurchaseOrderData,
  PurchaseOrderFilterParams,
  PurchaseOrderStatusTransitionRequest,
} from '../types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { purchaseOrderSchema, purchaseOrderDetailSchema } from '@/zod-schemas/orders';

export const purchaseApi = {
  list: async (params?: PurchaseOrderFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<PurchaseOrder>>(
      '/purchase-orders',
      params as Record<string, unknown>,
    );
    return validateResponse(paginatedDataSchema(purchaseOrderSchema), res.data);
  },

  /** Get purchase order detail — returns { order, items } structure. */
  get: async (id: number) => {
    const res = await apiClient.get<ApiResponse<{ order: PurchaseOrder; items: PurchaseOrderItem[] }>>(
      `/purchase-orders/${id}`,
    );
    return validateResponse(purchaseOrderDetailSchema, res.data) as { order: PurchaseOrder; items: PurchaseOrderItem[] };
  },

  create: async (data: CreatePurchaseOrderData) => {
    const res = await apiClient.post<ApiResponse<PurchaseOrder>>('/purchase-orders', data);
    return validateResponse(purchaseOrderSchema, res.data);
  },

  update: async (id: number, data: Partial<CreatePurchaseOrderData>) => {
    const res = await apiClient.put<ApiResponse<PurchaseOrder>>(`/purchase-orders/${id}`, data);
    return validateResponse(purchaseOrderSchema, res.data);
  },

  delete: async (id: number) => {
    await apiClient.delete(`/purchase-orders/${id}`);
  },

  approve: async (id: number, body?: { notes?: string }) => {
    const res = await apiClient.post<ApiResponse<PurchaseOrder>>(`/purchase-orders/${id}/approve`, body ?? {});
    return validateResponse(purchaseOrderSchema, res.data);
  },

  reject: async (id: number, body: { reason: string }) => {
    const res = await apiClient.post<ApiResponse<PurchaseOrder>>(`/purchase-orders/${id}/reject`, body);
    return validateResponse(purchaseOrderSchema, res.data);
  },

  linkInbound: async (id: number, inboundRecordId: number) => {
    const res = await apiClient.post<ApiResponse<PurchaseOrder>>(`/purchase-orders/${id}/link-inbound`, { inbound_record_id: inboundRecordId });
    return validateResponse(purchaseOrderSchema, res.data);
  },

  // 状态流转：pending → approved → received 等，具体流转由后端校验
  transition: async (id: number, data: PurchaseOrderStatusTransitionRequest) => {
    const res = await apiClient.post<ApiResponse<PurchaseOrder>>(
      `/purchase-orders/${id}/transition`,
      data,
    );
    return validateResponse(purchaseOrderSchema, res.data);
  },

  // 修改订单行项（数量、单价等）
  updateItem: async (orderId: number, itemId: number, data: Record<string, unknown>) => {
    const res = await apiClient.put<ApiResponse<PurchaseOrder>>(
      `/purchase-orders/${orderId}/items/${itemId}`,
      data,
    );
    return validateResponse(purchaseOrderSchema, res.data);
  },

  deleteItem: async (orderId: number, itemId: number) => {
    await apiClient.delete(`/purchase-orders/${orderId}/items/${itemId}`);
  },
};
