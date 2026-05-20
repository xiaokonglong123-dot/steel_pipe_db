import client from '../../../api/client';
import { ApiResponse, ApiListResponse } from '../../../shared/types';
import type { OrderStatus } from '../../../shared/types';

export interface SalesOrderItem {
  id?: string;
  sales_order_id?: string;
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  quantity: number;
  unit_price: number;
  subtotal?: number;
  delivered_qty?: number;
}

export interface OutboundRef {
  outbound_id: string;
  outbound_no: string;
}

export interface SalesOrder {
  id: string;
  order_no: string;
  customer_id: string;
  customer_name?: string;
  operator_name?: string;
  status: OrderStatus;
  total_amount: number;
  notes?: string;
  items?: SalesOrderItem[];
  created_at: string;
  updated_at: string;
  inbound_refs?: { inbound_id: string; inbound_no: string }[];
  outbound_refs?: OutboundRef[];
}

export interface SalesOrderFilter {
  page?: number;
  page_size?: number;
  status?: string;
  search?: string;
  date_from?: string;
  date_to?: string;
}

export interface SalesOrderCreatePayload {
  customer_id: string;
  notes?: string;
  items: Omit<SalesOrderItem, 'id' | 'sales_order_id' | 'subtotal' | 'delivered_qty'>[];
}

export interface AtpQuery {
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  quantity: number;
}

export interface AtpResult {
  available: boolean;
  available_qty: number;
  requested_qty: number;
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
}

const BASE = '/sales-orders';

export const salesApi = {
  list: (params?: SalesOrderFilter) =>
    client.get<ApiListResponse<SalesOrder>>(BASE, { params }),

  create: (data: SalesOrderCreatePayload) =>
    client.post<ApiResponse<SalesOrder>>(BASE, data),

  get: (id: string) =>
    client.get<ApiResponse<SalesOrder>>(`${BASE}/${id}`),

  approve: (id: string) =>
    client.put<ApiResponse<SalesOrder>>(`${BASE}/${id}/approve`),

  cancel: (id: string) =>
    client.put<ApiResponse<SalesOrder>>(`${BASE}/${id}/cancel`),

  checkAtp: (data: AtpQuery) =>
    client.post<ApiResponse<AtpResult>>(`${BASE}/atp`, data),
};
