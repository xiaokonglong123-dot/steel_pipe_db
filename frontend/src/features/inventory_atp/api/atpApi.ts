// Inventory ATP API — reservations, transfers, count templates
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';

export interface AtpRow {
  pipe_type: string;
  on_hand: string;
  reserved: string;
  available: string;
}

export interface AtpSlot {
  id: number;
  pipe_type: string;
  pipe_number: string | null;
  quantity_reserved: string;
  sales_order_id: number | null;
  status: string;
}

export interface InternalTransfer {
  id: number;
  transfer_no: string;
  from_location_id: number;
  to_location_id: number;
  pipe_number: string | null;
  quantity: string;
  status: string;
}

const atpRowSchema = z.object({
  pipe_type: z.string(), on_hand: z.string(), reserved: z.string(), available: z.string(),
}).passthrough();
const transferSchema = z.object({
  id: z.number(), transfer_no: z.string(), from_location_id: z.number(), to_location_id: z.number(),
  pipe_number: z.string().nullable(), quantity: z.string(), status: z.string(),
}).passthrough();
const arrayOf = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: z.array(item) }).passthrough();

export const atpApi = {
  overview: async () => {
    const res = await apiClient.get<ApiResponse<AtpRow[]>>('/inventory/atp/overview');
    return validateResponse(arrayOf(atpRowSchema), res.data).data;
  },
  reserve: async (data: { pipe_type: string; pipe_number?: string; quantity: number; sales_order_id?: number }) => {
    const res = await apiClient.post<ApiResponse<AtpSlot>>('/inventory/reservations', data);
    return res.data;
  },
  // (createTransfer unchanged)
  listTransfers: async () => {
    const res = await apiClient.get<ApiResponse<InternalTransfer[]>>('/inventory/transfers');
    return validateResponse(arrayOf(transferSchema), res.data).data;
  },
  createTransfer: async (data: { from_location_id: number; to_location_id: number; pipe_number: string; quantity: number }) => {
    const res = await apiClient.post<ApiResponse<InternalTransfer>>('/inventory/transfers', data);
    return validateResponse(transferSchema, res.data).data;
  },
  createCountTemplate: async (data: { name: string; location_ids: number[] }) => {
    const res = await apiClient.post<ApiResponse<unknown>>('/inventory/count-templates', data);
    return res.data;
  },
};
