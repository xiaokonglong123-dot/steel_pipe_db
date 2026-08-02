// 钢管管理 API — 无缝钢管、筛管、焊接管的 CRUD（API 5CT / API 5L 标准）
import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse, SeamlessPipe, ScreenPipe, WeldedPipe } from '@/types';
import type { CreateSeamlessPipeData, CreateScreenPipeData, CreateWeldedPipeData, PipeFilterParams } from '../types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { seamlessPipeSchema, screenPipeSchema, weldedPipeSchema } from '@/zod-schemas/core';

export const pipeApi = {
  getSeamlessPipes: async (params?: PipeFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<SeamlessPipe>>(
      '/seamless-pipes',
      params,
    );
    return validateResponse(paginatedDataSchema(seamlessPipeSchema), res.data);
  },

  getSeamlessPipe: async (id: number) => {
    const res = await apiClient.get<ApiResponse<SeamlessPipe>>(`/seamless-pipes/${id}`);
    return validateResponse(seamlessPipeSchema, res.data);
  },

  createSeamlessPipe: async (data: CreateSeamlessPipeData) => {
    const res = await apiClient.post<ApiResponse<SeamlessPipe>>('/seamless-pipes', data);
    return validateResponse(seamlessPipeSchema, res.data);
  },

  updateSeamlessPipe: async (id: number, data: Partial<CreateSeamlessPipeData>) => {
    const res = await apiClient.put<ApiResponse<SeamlessPipe>>(`/seamless-pipes/${id}`, data);
    return validateResponse(seamlessPipeSchema, res.data);
  },

  deleteSeamlessPipe: async (id: number) => {
    await apiClient.delete(`/seamless-pipes/${id}`);
  },

  getScreenPipes: async (params?: PipeFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<ScreenPipe>>('/screen-pipes', params);
    return validateResponse(paginatedDataSchema(screenPipeSchema), res.data);
  },

  getScreenPipe: async (id: number) => {
    const res = await apiClient.get<ApiResponse<ScreenPipe>>(`/screen-pipes/${id}`);
    return validateResponse(screenPipeSchema, res.data);
  },

  createScreenPipe: async (data: CreateScreenPipeData) => {
    const res = await apiClient.post<ApiResponse<ScreenPipe>>('/screen-pipes', data);
    return validateResponse(screenPipeSchema, res.data);
  },

  updateScreenPipe: async (id: number, data: Partial<CreateScreenPipeData>) => {
    const res = await apiClient.put<ApiResponse<ScreenPipe>>(`/screen-pipes/${id}`, data);
    return validateResponse(screenPipeSchema, res.data);
  },

  deleteScreenPipe: async (id: number) => {
    await apiClient.delete(`/screen-pipes/${id}`);
  },

  getWeldedPipes: async (params?: PipeFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<WeldedPipe>>('/welded-pipes', params);
    return validateResponse(paginatedDataSchema(weldedPipeSchema), res.data);
  },

  getWeldedPipe: async (id: number) => {
    const res = await apiClient.get<ApiResponse<WeldedPipe>>(`/welded-pipes/${id}`);
    return validateResponse(weldedPipeSchema, res.data);
  },

  createWeldedPipe: async (data: CreateWeldedPipeData) => {
    const res = await apiClient.post<ApiResponse<WeldedPipe>>('/welded-pipes', data);
    return validateResponse(weldedPipeSchema, res.data);
  },

  updateWeldedPipe: async (id: number, data: Partial<CreateWeldedPipeData>) => {
    const res = await apiClient.put<ApiResponse<WeldedPipe>>(`/welded-pipes/${id}`, data);
    return validateResponse(weldedPipeSchema, res.data);
  },

  deleteWeldedPipe: async (id: number) => {
    await apiClient.delete(`/welded-pipes/${id}`);
  },
};
