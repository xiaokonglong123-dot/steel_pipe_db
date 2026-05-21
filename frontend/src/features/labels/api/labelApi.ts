// 标签打印 API — 钢管标签、批量标签、质量标签、发货标签
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import type { PipeLabel, BatchLabelRequest, ShippingLabelRequest, LabelData } from '../types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';
import { pipeLabelSchema, labelDataSchema } from '@/zod-schemas/labels';

export const labelApi = {
  getPipeLabel: async (pipeType: string, pipeId: number) => {
    const res = await apiClient.get<ApiResponse<PipeLabel>>(`/labels/pipe/${pipeType}/${pipeId}`);
    return validateResponse(pipeLabelSchema, res.data.data);
  },

  createBatchLabels: async (data: BatchLabelRequest) => {
    const res = await apiClient.post<ApiResponse<LabelData[]>>('/labels/batch', data);
    return validateResponse(z.array(labelDataSchema), res.data.data);
  },

  getQualityLabel: async (certId: number) => {
    const res = await apiClient.get<ApiResponse<LabelData>>(`/labels/quality/${certId}`);
    return validateResponse(labelDataSchema, res.data.data);
  },

  createShippingLabel: async (data: ShippingLabelRequest) => {
    const res = await apiClient.post<ApiResponse<LabelData>>('/labels/shipping', data);
    return validateResponse(labelDataSchema, res.data.data);
  },
};
