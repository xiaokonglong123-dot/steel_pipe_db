import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import type { PipeLabel, BatchLabelRequest, ShippingLabelRequest, LabelData } from '../types';

export const labelApi = {
  getPipeLabel: async (pipeType: string, pipeId: number) => {
    const res = await apiClient.get<ApiResponse<PipeLabel>>(`/labels/pipe/${pipeType}/${pipeId}`);
    return res.data.data;
  },

  createBatchLabels: async (data: BatchLabelRequest) => {
    const res = await apiClient.post<ApiResponse<LabelData[]>>('/labels/batch', data);
    return res.data.data;
  },

  getQualityLabel: async (certId: number) => {
    const res = await apiClient.get<ApiResponse<LabelData>>(`/labels/quality/${certId}`);
    return res.data.data;
  },

  createShippingLabel: async (data: ShippingLabelRequest) => {
    const res = await apiClient.post<ApiResponse<LabelData>>('/labels/shipping', data);
    return res.data.data;
  },
};
