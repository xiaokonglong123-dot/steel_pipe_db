import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import type { BatchLabelRequest, ShippingLabelRequest } from '../types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';

export const labelApi = {
  getPipeLabel: async (pipeType: string, pipeId: number) => {
    const res = await apiClient.get<ApiResponse<string>>(`/labels/pipe/${pipeType}/${pipeId}`);
    return validateResponse(z.string(), res.data);
  },

  createBatchLabels: async (data: BatchLabelRequest) => {
    const res = await apiClient.post<ApiResponse<string>>('/labels/batch', data);
    return validateResponse(z.string(), res.data);
  },

  getQualityLabel: async (certId: number) => {
    const res = await apiClient.get<ApiResponse<string>>(`/labels/quality/${certId}`);
    return validateResponse(z.string(), res.data);
  },

  createShippingLabel: async (data: ShippingLabelRequest) => {
    const res = await apiClient.post<ApiResponse<string>>('/labels/shipping', data);
    return validateResponse(z.string(), res.data);
  },
};
