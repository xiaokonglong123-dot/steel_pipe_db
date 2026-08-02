// BI API
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';

export interface SalesTrendRow {
  month: string;
  status: string;
  order_count: number;
  total_amount: string;
}

export interface InventoryValueRow {
  pipe_type: string;
  on_hand: number;
}

export interface FinanceSummary {
  posted_entries: number;
  open_ar: string;
  open_ap: string;
  payment_count: number;
}

const trendSchema = z.object({
  month: z.string(), status: z.string(), order_count: z.number(), total_amount: z.string(),
}).passthrough();
const invSchema = z.object({ pipe_type: z.string(), on_hand: z.number() }).passthrough();
const finSchema = z.object({
  posted_entries: z.number(), open_ar: z.string(), open_ap: z.string(), payment_count: z.number(),
}).passthrough();
const arrayOf = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: z.array(item) }).passthrough();
const singleOf = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: item }).passthrough();

export const biApi = {
  salesTrend: async (months = 12): Promise<SalesTrendRow[]> => {
    const res = await apiClient.get<ApiResponse<SalesTrendRow[]>>('/bi/sales-trend', { months });
    return validateResponse(arrayOf(trendSchema), res.data).data;
  },
  inventoryValue: async (): Promise<InventoryValueRow[]> => {
    const res = await apiClient.get<ApiResponse<InventoryValueRow[]>>('/bi/inventory-value');
    return validateResponse(arrayOf(invSchema), res.data).data;
  },
  financeSummary: async (): Promise<FinanceSummary> => {
    const res = await apiClient.get<ApiResponse<FinanceSummary>>('/bi/finance-summary');
    return validateResponse(singleOf(finSchema), res.data).data;
  },
};
