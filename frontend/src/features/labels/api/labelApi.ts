import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import type { PipeLabel, BatchLabelRequest, ShippingLabelRequest, LabelData } from '../types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';
import { pipeLabelSchema, labelDataSchema } from '@/zod-schemas/labels';

export const labelApi = {
  getPipeLabel: async (pipeType: string, pipeId: number) => {
    const res = await apiClient.get<ApiResponse<PipeLabel>>(`/labels/pipe/${pipeType}/${pipeId}`);
    return validateResponse(res.data.data, pipeLabelSchema);
  },

  createBatchLabels: async (data: BatchLabelRequest) => {
    const res = await apiClient.post<ApiResponse<LabelData[]>>('/labels/batch', data);
    return validateResponse(res.data.data, z.array(labelDataSchema));
  },

  getQualityLabel: async (certId: number) => {
    const res = await apiClient.get<ApiResponse<LabelData>>(`/labels/quality/${certId}`);
    return validateResponse(res.data.data, labelDataSchema);
  },

  createShippingLabel: async (data: ShippingLabelRequest) => {
    const res = await apiClient.post<ApiResponse<LabelData>>('/labels/shipping', data);
    return validateResponse(res.data.data, labelDataSchema);
  },
};
