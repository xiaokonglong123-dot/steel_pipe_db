// Portal admin API — create portal accounts
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';

const accountSchema = z.object({
  id: z.number(), party_type: z.string(), party_id: z.number(), username: z.string(), is_active: z.boolean(),
}).passthrough();

export const portalApi = {
  createAccount: async (data: { party_type: string; party_id: number; username: string; password: string }) => {
    const res = await apiClient.post<ApiResponse<unknown>>('/portal/accounts', data);
    return validateResponse(accountSchema, res.data).data;
  },
};
