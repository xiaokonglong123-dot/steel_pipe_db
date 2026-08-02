// Sales CRM API — quotes, shipments
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';

export interface SalesQuote {
  id: number;
  quote_no: string;
  customer_id: number;
  total_amount: string;
  status: string;
}

export interface Shipment {
  id: number;
  shipment_no: string;
  sales_order_id: number;
  carrier: string | null;
  tracking_no: string | null;
  status: string;
}

const quoteSchema = z.object({
  id: z.number(), quote_no: z.string(), customer_id: z.number(), total_amount: z.string(), status: z.string(),
}).passthrough();
const shipmentSchema = z.object({
  id: z.number(), shipment_no: z.string(), sales_order_id: z.number(), carrier: z.string().nullable(),
  tracking_no: z.string().nullable(), status: z.string(),
}).passthrough();
const arrayOf = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: z.array(item) }).passthrough();

export const salesCrmApi = {
  listQuotes: async () => {
    const res = await apiClient.get<ApiResponse<SalesQuote[]>>('/sales-quotes');
    return validateResponse(arrayOf(quoteSchema), res.data).data;
  },
  createQuote: async (data: { customer_id: number; total_amount: number; items: unknown[] }) => {
    const res = await apiClient.post<ApiResponse<SalesQuote>>('/sales-quotes', data);
    return validateResponse(quoteSchema, res.data).data;
  },
  updateQuoteStatus: async (id: number, status: string) => {
    const res = await apiClient.put<ApiResponse<SalesQuote>>(`/sales-quotes/${id}/status`, { status });
    return validateResponse(quoteSchema, res.data).data;
  },
  convertQuote: async (id: number) => {
    const res = await apiClient.post<ApiResponse<{ order_id: number }>>(`/sales-quotes/${id}/convert`);
    return res.data;
  },
  listShipments: async () => {
    const res = await apiClient.get<ApiResponse<Shipment[]>>('/shipments');
    return validateResponse(arrayOf(shipmentSchema), res.data).data;
  },
  createShipment: async (data: { sales_order_id: number; items: unknown[] }) => {
    const res = await apiClient.post<ApiResponse<Shipment>>('/shipments', data);
    return validateResponse(shipmentSchema, res.data).data;
  },
  updateShipmentStatus: async (id: number, status: string) => {
    const res = await apiClient.put<ApiResponse<Shipment>>(`/shipments/${id}/status`, { status });
    return validateResponse(shipmentSchema, res.data).data;
  },
};
