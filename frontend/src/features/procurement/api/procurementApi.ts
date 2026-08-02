// Procurement API — requisitions, receipts, quotes
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';

export interface Requisition {
  id: number;
  req_no: string;
  title: string;
  status: string;
  items_json?: unknown;
}

export interface SupplierQuote {
  id: number;
  quote_no: string;
  supplier_id: number;
  title: string | null;
  total_amount: string;
  status: string;
}

const reqSchema = z.object({
  id: z.number(), req_no: z.string(), title: z.string(), status: z.string(), items_json: z.unknown(),
}).passthrough();
const quoteSchema = z.object({
  id: z.number(), quote_no: z.string(), supplier_id: z.number(), title: z.string().nullable(),
  total_amount: z.string(), status: z.string(),
}).passthrough();
const arrayOf = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: z.array(item) }).passthrough();

export const procurementApi = {
  listRequisitions: async (status?: string) => {
    const res = await apiClient.get<ApiResponse<Requisition[]>>('/purchase-requisitions', { status });
    return validateResponse(arrayOf(reqSchema), res.data).data;
  },
  createRequisition: async (data: { title: string; items: unknown[] }) => {
    const res = await apiClient.post<ApiResponse<Requisition>>('/purchase-requisitions', data);
    return validateResponse(reqSchema, res.data).data;
  },
  updateRequisitionStatus: async (id: number, status: string) => {
    const res = await apiClient.put<ApiResponse<Requisition>>(`/purchase-requisitions/${id}`, { status });
    return validateResponse(reqSchema, res.data).data;
  },
  listQuotes: async () => {
    const res = await apiClient.get<ApiResponse<SupplierQuote[]>>('/supplier-quotes');
    return validateResponse(arrayOf(quoteSchema), res.data).data;
  },
  createQuote: async (data: { supplier_id: number; title?: string; total_amount: number; items: unknown[] }) => {
    const res = await apiClient.post<ApiResponse<SupplierQuote>>('/supplier-quotes', data);
    return validateResponse(quoteSchema, res.data).data;
  },
  updateQuoteStatus: async (id: number, status: string) => {
    const res = await apiClient.put<ApiResponse<SupplierQuote>>(`/supplier-quotes/${id}/status`, { status });
    return validateResponse(quoteSchema, res.data).data;
  },
};
