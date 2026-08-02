// Assets API
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';

export interface FixedAsset {
  id: number;
  asset_no: string;
  name: string;
  category: string;
  purchase_cost: string;
  current_value: string;
  status: string;
}

const assetSchema = z.object({
  id: z.number(), asset_no: z.string(), name: z.string(), category: z.string(),
  purchase_cost: z.string(), current_value: z.string(), status: z.string(),
}).passthrough();
const arrayOf = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: z.array(item) }).passthrough();

export const assetsApi = {
  list: async () => {
    const res = await apiClient.get<ApiResponse<FixedAsset[]>>('/assets');
    return validateResponse(arrayOf(assetSchema), res.data).data;
  },
  create: async (data: { name: string; purchase_date: string; purchase_cost: number; useful_life_months?: number }) => {
    const res = await apiClient.post<ApiResponse<FixedAsset>>('/assets', data);
    return validateResponse(assetSchema, res.data).data;
  },
  depreciate: async (id: number, period: string) => {
    const res = await apiClient.post<ApiResponse<{ amount: string }>>(`/assets/${id}/depreciate`, { period });
    return res.data;
  },
  dispose: async (id: number) => {
    const res = await apiClient.post<ApiResponse<FixedAsset>>(`/assets/${id}/dispose`);
    return validateResponse(assetSchema, res.data).data;
  },
};
