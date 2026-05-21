// 钢管管理 API — 无缝钢管和筛管的 CRUD（API 5CT 标准）
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
    return validateResponse(paginatedDataSchema(seamlessPipeSchema), res.data.data);
  },

  getSeamlessPipe: async (id: number) => {
    const res = await apiClient.get<ApiResponse<SeamlessPipe>>(`/seamless-pipes/${id}`);
    return validateResponse(seamlessPipeSchema, res.data.data);
  },

  createSeamlessPipe: async (data: CreateSeamlessPipeData) => {
    const res = await apiClient.post<ApiResponse<SeamlessPipe>>('/seamless-pipes', data);
    return validateResponse(seamlessPipeSchema, res.data.data);
  },

  updateSeamlessPipe: async (id: number, data: Partial<CreateSeamlessPipeData>) => {
    const res = await apiClient.put<ApiResponse<SeamlessPipe>>(`/seamless-pipes/${id}`, data);
    return validateResponse(seamlessPipeSchema, res.data.data);
  },

  deleteSeamlessPipe: async (id: number) => {
    await apiClient.delete(`/seamless-pipes/${id}`);
  },

  getScreenPipes: async (params?: PipeFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<ScreenPipe>>('/screen-pipes', { params });
    return validateResponse(paginatedDataSchema(screenPipeSchema), res.data.data);
  },

  getScreenPipe: async (id: number) => {
    const res = await apiClient.get<ApiResponse<ScreenPipe>>(`/screen-pipes/${id}`);
    return validateResponse(screenPipeSchema, res.data.data);
  },

  createScreenPipe: async (data: CreateScreenPipeData) => {
    const res = await apiClient.post<ApiResponse<ScreenPipe>>('/screen-pipes', data);
    return validateResponse(screenPipeSchema, res.data.data);
  },

  updateScreenPipe: async (id: number, data: Partial<CreateScreenPipeData>) => {
    const res = await apiClient.put<ApiResponse<ScreenPipe>>(`/screen-pipes/${id}`, data);
    return validateResponse(screenPipeSchema, res.data.data);
  },

  deleteScreenPipe: async (id: number) => {
    await apiClient.delete(`/screen-pipes/${id}`);
  },
};
