import client from '../../../api/client';
import { ApiResponse, ApiListResponse } from '../../../shared/types';

export interface LabelField {
  key: string;
  label: string;
  font_size: number;
  x: number;
  y: number;
  width: number;
  height: number;
  align: 'left' | 'center' | 'right';
}

export interface LabelTemplate {
  id: string;
  name: string;
  width_mm: number;
  height_mm: number;
  fields: LabelField[];
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface LabelPrintRequest {
  pipe_ids: string[];
  template_id: string;
  copies: number;
}

export interface LabelPrintResponse {
  job_id: string;
  url: string;
}

export interface PrintJob {
  id: string;
  job_id: string;
  template_name: string;
  pipe_count: number;
  copies: number;
  status: string;
  created_at: string;
}

export interface LabelFilter {
  page?: number;
  page_size?: number;
}

export const labelApi = {
  listTemplates: (params?: LabelFilter) =>
    client.get<ApiListResponse<LabelTemplate>>('/label-templates', { params }),

  getTemplate: (id: string) =>
    client.get<ApiResponse<LabelTemplate>>(`/label-templates/${id}`),

  createTemplate: (data: Partial<LabelTemplate>) =>
    client.post<ApiResponse<LabelTemplate>>('/label-templates', data),

  updateTemplate: (id: string, data: Partial<LabelTemplate>) =>
    client.put<ApiResponse<LabelTemplate>>(`/label-templates/${id}`, data),

  deleteTemplate: (id: string) =>
    client.delete(`/label-templates/${id}`),

  printLabels: (data: LabelPrintRequest) =>
    client.post<ApiResponse<LabelPrintResponse>>('/labels/generate', data),

  listPrintJobs: (params?: LabelFilter) =>
    client.get<ApiListResponse<PrintJob>>('/labels/print-history', { params }),
};
