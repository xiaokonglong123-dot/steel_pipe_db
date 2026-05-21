import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse, SeamlessPipe, ScreenPipe } from '@/types';
import type { CreateSeamlessPipeData, CreateScreenPipeData, PipeFilterParams } from '../types';

export const pipeApi = {
  getSeamlessPipes: async (params?: PipeFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<SeamlessPipe>>(
      '/seamless-pipes',
      { params },
    );
    return res.data.data;
  },

  getSeamlessPipe: async (id: number) => {
    const res = await apiClient.get<ApiResponse<SeamlessPipe>>(`/seamless-pipes/${id}`);
    return res.data.data;
  },

  createSeamlessPipe: async (data: CreateSeamlessPipeData) => {
    const res = await apiClient.post<ApiResponse<SeamlessPipe>>('/seamless-pipes', data);
    return res.data.data;
  },

  updateSeamlessPipe: async (id: number, data: Partial<CreateSeamlessPipeData>) => {
    const res = await apiClient.put<ApiResponse<SeamlessPipe>>(`/seamless-pipes/${id}`, data);
    return res.data.data;
  },

  deleteSeamlessPipe: async (id: number) => {
    await apiClient.delete(`/seamless-pipes/${id}`);
  },

  getScreenPipes: async (params?: PipeFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<ScreenPipe>>('/screen-pipes', { params });
    return res.data.data;
  },

  getScreenPipe: async (id: number) => {
    const res = await apiClient.get<ApiResponse<ScreenPipe>>(`/screen-pipes/${id}`);
    return res.data.data;
  },

  createScreenPipe: async (data: CreateScreenPipeData) => {
    const res = await apiClient.post<ApiResponse<ScreenPipe>>('/screen-pipes', data);
    return res.data.data;
  },

  updateScreenPipe: async (id: number, data: Partial<CreateScreenPipeData>) => {
    const res = await apiClient.put<ApiResponse<ScreenPipe>>(`/screen-pipes/${id}`, data);
    return res.data.data;
  },

  deleteScreenPipe: async (id: number) => {
    await apiClient.delete(`/screen-pipes/${id}`);
  },
};
