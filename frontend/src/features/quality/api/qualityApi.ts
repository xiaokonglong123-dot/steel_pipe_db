import client from '../../../api/client';
import { ApiResponse, ApiListResponse } from '../../../shared/types';
import type { CertResult, PipeCategory } from '../../../shared/types';

export interface QualityCert {
  id: string;
  cert_no: string;
  pipe_type: PipeCategory | 'plain_end';
  pipe_id: string;
  pipe_number?: string;
  inspect_date: string;
  inspector: string;
  agency?: string;
  result: CertResult;
  items_json?: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}

export interface QualityCertItem {
  item_no: number;
  test_item: string;
  specification: string;
  measured_value: string;
  result: CertResult;
  remark?: string;
}

export interface Api5ctGradeRef {
  id: number;
  grade: string;
  pipe_type: string;
  od_min: number;
  od_max: number;
  wt_min: number;
  wt_max: number;
  yield_min: number;
  yield_max: number;
  tensile_min: number;
  hardness_max?: number;
}

export interface QualityCertListParams {
  page?: number;
  page_size?: number;
  pipe_type?: string;
  result?: string;
  cert_no?: string;
  date_from?: string;
  date_to?: string;
}

export interface QualityCertListResponse {
  success: boolean;
  data: QualityCert[];
  meta: {
    page: number;
    page_size: number;
    total: number;
    total_pages: number;
  };
  request_id: string;
}

export const qualityApi = {
  listCert: (pipeType: string, params?: QualityCertListParams) =>
    client.get<ApiListResponse<QualityCert>>(
      `/quality/certs`,
      { params }
    ),

  getCert: (pipeType: string, id: string) =>
    client.get<ApiResponse<QualityCert>>(
      `/quality/certs/${id}`
    ),

  createCert: (pipeType: string, data: Partial<QualityCert>) =>
    client.post<ApiResponse<QualityCert>>(
      `/quality/certs`,
      data
    ),

  updateCert: (pipeType: string, id: string, data: Partial<QualityCert>) =>
    client.put<ApiResponse<QualityCert>>(
      `/quality/certs/${id}`,
      data
    ),

  deleteCert: (pipeType: string, id: string) =>
    client.delete(`/quality/certs/${id}`),

  listGradeRefs: (params?: { pipe_type?: string }) =>
    client.get<ApiListResponse<Api5ctGradeRef>>('/quality/api5ct/grades', {
      params,
    }),
};
