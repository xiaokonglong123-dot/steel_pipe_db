import client from '../../../api/client';
import { ApiResponse, ApiListResponse } from '../../../shared/types';

export interface InboundRecord {
  id: string;
  inbound_no: string;
  inbound_type: string;
  supplier_id?: string;
  order_id?: string;
  operator_id?: string;
  total_items: number;
  notes?: string;
  created_at: string;
}

export interface InboundItem {
  id: string;
  inbound_id: string;
  pipe_type: string;
  pipe_id: string;
  confirmed: boolean;
  notes?: string;
}

export interface OutboundRecord {
  id: string;
  outbound_no: string;
  outbound_type: string;
  customer_id?: string;
  order_id?: string;
  operator_id?: string;
  total_items: number;
  notes?: string;
  created_at: string;
}

export interface OutboundItem {
  id: string;
  outbound_id: string;
  pipe_type: string;
  pipe_id: string;
  confirmed: boolean;
  notes?: string;
}

export interface StockSummary {
  total_in_stock: number;
  seamless_count: number;
  screen_count: number;
  in_stock_count: number;
  outbound_count: number;
  scrapped_count: number;
}

export interface InboundFilter {
  page?: number;
  page_size?: number;
  inbound_no?: string;
  inbound_type?: string;
  date_from?: string;
  date_to?: string;
}

export interface OutboundFilter {
  page?: number;
  page_size?: number;
  outbound_no?: string;
  outbound_type?: string;
  date_from?: string;
  date_to?: string;
}

export interface CreateInboundPayload {
  inbound_type: string;
  pipe_ids: string[];
  notes?: string;
}

export interface CreateOutboundPayload {
  outbound_type: string;
  pipe_ids: string[];
  notes?: string;
}

export interface PipeOption {
  id: string;
  pipe_number: string;
  pipe_type: 'seamless' | 'screen';
  grade: string;
  od: number;
  wt: number;
  status: string;
}

export const inventoryApi = {
  listInbound: (params?: InboundFilter) =>
    client.get<ApiListResponse<InboundRecord>>('/inventory/inbound', { params }),

  createInbound: (data: CreateInboundPayload) =>
    client.post<ApiResponse<InboundRecord>>('/inventory/inbound', data),

  getInbound: (id: string) =>
    client.get<ApiResponse<InboundRecord>>(`/inventory/inbound/${id}`),

  listOutbound: (params?: OutboundFilter) =>
    client.get<ApiListResponse<OutboundRecord>>('/inventory/outbound', { params }),

  createOutbound: (data: CreateOutboundPayload) =>
    client.post<ApiResponse<OutboundRecord>>('/inventory/outbound', data),

  getOutbound: (id: string) =>
    client.get<ApiResponse<OutboundRecord>>(`/inventory/outbound/${id}`),

  getStockSummary: () =>
    client.get<ApiResponse<StockSummary>>('/inventory/stock'),
};
