import client from '../../../api/client';
import { ApiResponse, ApiListResponse } from '../../../shared/types';

export interface SeamlessPipe {
  id: string;
  pipe_number: string;
  grade: string;
  od: number;
  wt: number;
  length: number;
  weight: number;
  connection_type?: string;
  heat_number?: string;
  production_date?: string;
  status: string;
  location?: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}

export interface ScreenPipe {
  id: string;
  pipe_number: string;
  grade: string;
  od: number;
  wt: number;
  length: number;
  weight: number;
  screen_type: string;
  slot_width?: number;
  open_area?: number;
  connection_type?: string;
  heat_number?: string;
  production_date?: string;
  status: string;
  location?: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}

export interface PipeFilter {
  page?: number;
  page_size?: number;
  grade?: string;
  status?: string;
  search?: string;
}

export const pipeApi = {
  listSeamless: (params?: PipeFilter) =>
    client.get<ApiListResponse<SeamlessPipe>>('/seamless-pipes', { params }),

  createSeamless: (data: Partial<SeamlessPipe>) =>
    client.post<ApiResponse<SeamlessPipe>>('/seamless-pipes', data),

  getSeamless: (id: string) =>
    client.get<ApiResponse<SeamlessPipe>>(`/seamless-pipes/${id}`),

  updateSeamless: (id: string, data: Partial<SeamlessPipe>) =>
    client.put<ApiResponse<SeamlessPipe>>(`/seamless-pipes/${id}`, data),

  deleteSeamless: (id: string) =>
    client.delete(`/seamless-pipes/${id}`),

  listScreen: (params?: PipeFilter) =>
    client.get<ApiListResponse<ScreenPipe>>('/screen-pipes', { params }),

  createScreen: (data: Partial<ScreenPipe>) =>
    client.post<ApiResponse<ScreenPipe>>('/screen-pipes', data),

  getScreen: (id: string) =>
    client.get<ApiResponse<ScreenPipe>>(`/screen-pipes/${id}`),

  updateScreen: (id: string, data: Partial<ScreenPipe>) =>
    client.put<ApiResponse<ScreenPipe>>(`/screen-pipes/${id}`, data),

  deleteScreen: (id: string) =>
    client.delete(`/screen-pipes/${id}`),
};
