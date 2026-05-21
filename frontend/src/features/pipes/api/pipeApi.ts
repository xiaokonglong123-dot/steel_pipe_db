import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse, SeamlessPipe, ScreenPipe } from '@/types';
import type { CreateSeamlessPipeData, CreateScreenPipeData, PipeFilterParams } from '../types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { seamlessPipeSchema, screenPipeSchema } from '@/zod-schemas/core';

export const pipeApi = {
  getSeamlessPipes: async (params?: PipeFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<SeamlessPipe>>(
      '/seamless-pipes',
      { params },
    );
    return validateResponse(res.data.data, paginatedDataSchema(seamlessPipeSchema));
  },

  getSeamlessPipe: async (id: number) => {
    const res = await apiClient.get<ApiResponse<SeamlessPipe>>(`/seamless-pipes/${id}`);
    return validateResponse(res.data.data, seamlessPipeSchema);
  },

  createSeamlessPipe: async (data: CreateSeamlessPipeData) => {
    const res = await apiClient.post<ApiResponse<SeamlessPipe>>('/seamless-pipes', data);
    return validateResponse(res.data.data, seamlessPipeSchema);
  },

  updateSeamlessPipe: async (id: number, data: Partial<CreateSeamlessPipeData>) => {
    const res = await apiClient.put<ApiResponse<SeamlessPipe>>(`/seamless-pipes/${id}`, data);
    return validateResponse(res.data.data, seamlessPipeSchema);
  },

  deleteSeamlessPipe: async (id: number) => {
    await apiClient.delete(`/seamless-pipes/${id}`);
  },

  getScreenPipes: async (params?: PipeFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<ScreenPipe>>('/screen-pipes', { params });
    return validateResponse(res.data.data, paginatedDataSchema(screenPipeSchema));
  },

  getScreenPipe: async (id: number) => {
    const res = await apiClient.get<ApiResponse<ScreenPipe>>(`/screen-pipes/${id}`);
    return validateResponse(res.data.data, screenPipeSchema);
  },

  createScreenPipe: async (data: CreateScreenPipeData) => {
    const res = await apiClient.post<ApiResponse<ScreenPipe>>('/screen-pipes', data);
    return validateResponse(res.data.data, screenPipeSchema);
  },

  updateScreenPipe: async (id: number, data: Partial<CreateScreenPipeData>) => {
    const res = await apiClient.put<ApiResponse<ScreenPipe>>(`/screen-pipes/${id}`, data);
    return validateResponse(res.data.data, screenPipeSchema);
  },

  deleteScreenPipe: async (id: number) => {
    await apiClient.delete(`/screen-pipes/${id}`);
  },
};
