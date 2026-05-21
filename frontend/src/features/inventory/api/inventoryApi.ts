// 库存管理 API — 入库/出库/库位/盘点/库存查询，含审批流
import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { z } from 'zod';
import {
  inboundRecordSchema,
  inboundDetailSchema,
  outboundRecordSchema,
  outboundDetailSchema,
  locationSchema,
  inventoryLogSchema,
  inventoryCheckRecordSchema,
  inventoryCheckItemSchema,
  checkDetailSchema,
  pipeSearchResultSchema,
} from '@/zod-schemas/inventory';

// ━━━ Types ━━━

export interface InboundRecord {
  id: number;
  inbound_no: string;
  inbound_type: string;
  order_id?: number;
  supplier_id?: number;
  notes?: string;
  approval_status: string;
  handled_by?: number;
  handled_at?: string;
  created_at: string;
  updated_at: string;
}

export interface InboundItem {
  id: number;
  inbound_id: number;
  pipe_type: string;
  pipe_id: number;
  created_at: string;
}

export interface OutboundRecord {
  id: number;
  outbound_no: string;
  outbound_type: string;
  order_id?: number;
  customer_id?: number;
  notes?: string;
  approval_status: string;
  handled_by?: number;
  handled_at?: string;
  created_at: string;
  updated_at: string;
}

export interface OutboundItem {
  id: number;
  outbound_id: number;
  pipe_type: string;
  pipe_id: number;
  created_at: string;
}

export interface Location {
  id: number;
  zone_code: string;
  shelf_code: string;
  level_code: string;
  full_code: string;
  description?: string;
  capacity?: number;
  used_count: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface InventoryLog {
  id: number;
  pipe_type: string;
  pipe_id: number;
  change_type: string;
  ref_type?: string;
  ref_id?: number;
  from_location_id?: number;
  to_location_id?: number;
  notes?: string;
  created_by?: number;
  created_at: string;
}

export interface InventoryCheckRecord {
  id: number;
  check_no: string;
  location_id?: number;
  status: string;
  notes?: string;
  created_by?: number;
  created_at: string;
  updated_at: string;
}

export interface InventoryCheckItem {
  id: number;
  check_id: number;
  pipe_type: string;
  pipe_id: number;
  expected_status: string;
  found_status?: string;
  is_match?: boolean;
  notes?: string;
  created_at: string;
}

export interface InboundDetail {
  record: InboundRecord;
  items: InboundItem[];
}

export interface OutboundDetail {
  record: OutboundRecord;
  items: OutboundItem[];
}

export interface CheckDetail {
  record: InventoryCheckRecord;
  items: InventoryCheckItem[];
}

// ━━━ Request types ━━━

export interface CreateInboundData {
  inbound_type: string;
  order_id?: number;
  supplier_id?: number;
  notes?: string;
  pipes: { pipe_type: string; pipe_id: number }[];
}

export interface CreateOutboundData {
  outbound_type: string;
  order_id?: number;
  customer_id?: number;
  notes?: string;
  pipes: { pipe_type: string; pipe_id: number }[];
}

export interface CreateLocationData {
  zone_code: string;
  shelf_code: string;
  level_code: string;
  description?: string;
  capacity?: number;
}

export interface UpdateLocationData {
  description?: string;
  capacity?: number;
  is_active?: boolean;
}

export interface CreateCheckData {
  location_id?: number;
  notes?: string;
}

export interface SubmitCheckItemData {
  found_status: string;
  notes?: string;
}

export interface ListFilter {
  page?: number;
  page_size?: number;
  sort_by?: string;
  sort_order?: string;
  q?: string;
}

export interface InboundFilter extends ListFilter {
  inbound_type?: string;
  approval_status?: string;
}

export interface OutboundFilter extends ListFilter {
  outbound_type?: string;
  approval_status?: string;
}

export interface StockFilter {
  page?: number;
  page_size?: number;
  grade?: string;
  pipe_type?: string;
  location_id?: number;
  q?: string;
}

export interface LocationFilter {
  page?: number;
  page_size?: number;
  active_only?: boolean;
}

export interface CheckFilter {
  page?: number;
  page_size?: number;
}

// ━━━ Inbound API ━━━

export const inboundApi = {
  list: async (params?: InboundFilter) => {
    const res = await apiClient.get<PaginatedResponse<InboundRecord>>('/inbound-records', { params });
    return validateResponse(paginatedDataSchema(inboundRecordSchema), res.data.data);
  },

  get: async (id: number) => {
    const res = await apiClient.get<ApiResponse<InboundDetail>>(`/inbound-records/${id}`);
    return validateResponse(inboundDetailSchema, res.data.data);
  },

  create: async (data: CreateInboundData) => {
    const res = await apiClient.post<ApiResponse<InboundRecord>>('/inbound-records', data);
    return validateResponse(inboundRecordSchema, res.data.data);
  },

  approve: async (id: number, reason?: string) => {
    const res = await apiClient.post<ApiResponse<string>>(`/inbound-records/${id}/approve`, { reason });
    validateResponse(z.string(), res.data.data);
    return res.data;
  },

  reject: async (id: number, reason: string) => {
    const res = await apiClient.post<ApiResponse<string>>(`/inbound-records/${id}/reject`, { reason });
    validateResponse(z.string(), res.data.data);
    return res.data;
  },

  delete: async (id: number) => {
    await apiClient.delete(`/inbound-records/${id}`);
  },
};

// ━━━ Outbound API ━━━

export const outboundApi = {
  list: async (params?: OutboundFilter) => {
    const res = await apiClient.get<PaginatedResponse<OutboundRecord>>('/outbound-records', { params });
    return validateResponse(paginatedDataSchema(outboundRecordSchema), res.data.data);
  },

  get: async (id: number) => {
    const res = await apiClient.get<ApiResponse<OutboundDetail>>(`/outbound-records/${id}`);
    return validateResponse(outboundDetailSchema, res.data.data);
  },

  create: async (data: CreateOutboundData) => {
    const res = await apiClient.post<ApiResponse<OutboundRecord>>('/outbound-records', data);
    return validateResponse(outboundRecordSchema, res.data.data);
  },

  approve: async (id: number, reason?: string) => {
    const res = await apiClient.post<ApiResponse<string>>(`/outbound-records/${id}/approve`, { reason });
    validateResponse(z.string(), res.data.data);
    return res.data;
  },

  reject: async (id: number, reason: string) => {
    const res = await apiClient.post<ApiResponse<string>>(`/outbound-records/${id}/reject`, { reason });
    validateResponse(z.string(), res.data.data);
    return res.data;
  },

  delete: async (id: number) => {
    await apiClient.delete(`/outbound-records/${id}`);
  },
};

// ━━━ Inventory API ━━━

export const inventoryApi = {
  queryStock: async (params?: StockFilter) => {
    const res = await apiClient.get<PaginatedResponse<Record<string, unknown>>>('/inventory', { params });
    return res.data.data;
  },

  queryLogs: async (params?: StockFilter) => {
    const res = await apiClient.get<PaginatedResponse<InventoryLog>>('/inventory/logs', { params });
    return validateResponse(paginatedDataSchema(inventoryLogSchema), res.data.data);
  },

  tracePipe: async (pipeType: string, pipeId: number) => {
    const res = await apiClient.get<ApiResponse<Record<string, unknown>>>(`/trace/pipe/${pipeType}/${pipeId}`);
    return res.data.data;
  },

  traceHeat: async (heatNumber: string) => {
    const res = await apiClient.get<ApiResponse<unknown[]>>(`/trace/heat-number/${heatNumber}`);
    return res.data.data;
  },

  traceOrder: async (orderType: string, orderId: number) => {
    const res = await apiClient.get<ApiResponse<unknown>>(`/trace/order/${orderType}/${orderId}`);
    return res.data.data;
  },
};

// ━━━ Location API ━━━

export const locationApi = {
  list: async (params?: LocationFilter) => {
    const res = await apiClient.get<PaginatedResponse<Location>>('/locations', { params });
    return validateResponse(paginatedDataSchema(locationSchema), res.data.data);
  },

  get: async (id: number) => {
    const res = await apiClient.get<ApiResponse<Location>>(`/locations/${id}`);
    return validateResponse(locationSchema, res.data.data);
  },

  create: async (data: CreateLocationData) => {
    const res = await apiClient.post<ApiResponse<Location>>('/locations', data);
    return validateResponse(locationSchema, res.data.data);
  },

  update: async (id: number, data: UpdateLocationData) => {
    const res = await apiClient.put<ApiResponse<Location>>(`/locations/${id}`, data);
    return validateResponse(locationSchema, res.data.data);
  },

  delete: async (id: number) => {
    await apiClient.delete(`/locations/${id}`);
  },
};

// ━━━ Check API ━━━

export const checkApi = {
  list: async (params?: CheckFilter) => {
    const res = await apiClient.get<PaginatedResponse<InventoryCheckRecord>>('/inventory/checks', { params });
    return validateResponse(paginatedDataSchema(inventoryCheckRecordSchema), res.data.data);
  },

  get: async (id: number) => {
    const res = await apiClient.get<ApiResponse<CheckDetail>>(`/inventory/checks/${id}`);
    return validateResponse(checkDetailSchema, res.data.data);
  },

  create: async (data: CreateCheckData) => {
    const res = await apiClient.post<ApiResponse<InventoryCheckRecord>>('/inventory/checks', data);
    return validateResponse(inventoryCheckRecordSchema, res.data.data);
  },

  submitItem: async (checkId: number, itemId: number, data: SubmitCheckItemData) => {
    const res = await apiClient.put<ApiResponse<InventoryCheckItem>>(
      `/inventory/checks/${checkId}/items/${itemId}`,
      data,
    );
    return validateResponse(inventoryCheckItemSchema, res.data.data);
  },
};

// ━━━ Pipe search (for modal selection) ━━━

export interface PipeSearchResult {
  id: number;
  pipe_type: string;
  pipe_number: string;
  grade: string;
  od: number;
  wt: number;
  status: string;
  location_id?: number;
}

export const pipeSearchApi = {
  search: async (params?: { q?: string; pipe_type?: string; status?: string; limit?: number }) => {
    const res = await apiClient.get<ApiResponse<PipeSearchResult[]>>('/pipes/search', { params });
    return validateResponse(z.array(pipeSearchResultSchema), res.data.data);
  },
};
