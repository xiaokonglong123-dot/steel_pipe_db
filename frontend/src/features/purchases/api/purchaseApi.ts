import client from '../../../api/client';
import { ApiResponse, ApiListResponse } from '../../../shared/types';
import type { OrderStatus } from '../../../shared/types';

export interface PurchaseOrderItem {
  id?: string;
  purchase_order_id?: string;
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  quantity: number;
  unit_price: number;
  subtotal?: number;
  received_qty?: number;
}

export interface InboundRef {
  inbound_id: string;
  inbound_no: string;
}

export interface PurchaseOrder {
  id: string;
  order_no: string;
  supplier_id: string;
  supplier_name?: string;
  operator_name?: string;
  status: OrderStatus;
  total_amount: number;
  notes?: string;
  items?: PurchaseOrderItem[];
  created_at: string;
  updated_at: string;
  inbound_refs?: InboundRef[];
  outbound_refs?: InboundRef[];
}

export interface PurchaseOrderFilter {
  page?: number;
  page_size?: number;
  status?: string;
  search?: string;
  date_from?: string;
  date_to?: string;
}

export interface PurchaseOrderCreatePayload {
  supplier_id: string;
  notes?: string;
  items: Omit<PurchaseOrderItem, 'id' | 'purchase_order_id' | 'subtotal' | 'received_qty'>[];
}

const BASE = '/purchase-orders';

export const purchaseApi = {
  list: (params?: PurchaseOrderFilter) =>
    client.get<ApiListResponse<PurchaseOrder>>(BASE, { params }),

  create: (data: PurchaseOrderCreatePayload) =>
    client.post<ApiResponse<PurchaseOrder>>(BASE, data),

  get: (id: string) =>
    client.get<ApiResponse<PurchaseOrder>>(`${BASE}/${id}`),

  approve: (id: string) =>
    client.put<ApiResponse<PurchaseOrder>>(`${BASE}/${id}/approve`),

  cancel: (id: string) =>
    client.put<ApiResponse<PurchaseOrder>>(`${BASE}/${id}/cancel`),
};
