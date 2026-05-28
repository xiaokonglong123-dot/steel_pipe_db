/**
 * Data IO API — Excel/CSV import, export, template download, and operation logs.
 *
 * Backend endpoints:
 * - POST   /api/v1/data/{entity_type}/import       — upload file for import
 * - GET    /api/v1/data/{entity_type}/export         — download exported data
 * - GET    /api/v1/data/{entity_type}/template       — download blank import template
 * - GET    /api/v1/data/logs                         — paginated operation logs
 */
import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';

// ━━━ Types ━━━

/** Import result returned after a successful import operation. */
export interface ImportResult {
  entity_type: string;
  imported_count: number;
  failed_count: number;
  errors: string[];
}

/** Operation log entry for import/export audit trail. */
export interface OperationLog {
  id: number;
  user_id: number | null;
  username: string | null;
  action: string;
  entity_type: string;
  entity_id: number | null;
  details: string | null;
  ip_address: string | null;
  created_at: string;
}

/** Supported entity types for import/export. */
export const ENTITY_TYPES = [
  { value: 'seamless_pipes', label: '无缝管' },
  { value: 'screen_pipes', label: '筛管' },
  { value: 'inventory', label: '库存' },
  { value: 'purchase_orders', label: '采购订单' },
  { value: 'sales_orders', label: '销售订单' },
  { value: 'quality_certs', label: '质量证书' },
] as const;

export type EntityType = (typeof ENTITY_TYPES)[number]['value'];

// ━━━ API Functions ━━━

export const dataIoApi = {
  /** Upload a file for import. Returns import stats. */
  importData: async (entityType: string, file: File) => {
    const formData = new FormData();
    formData.append('file', file);
    const res = await apiClient.post<ApiResponse<ImportResult>>(
      `/data/${entityType}/import`,
      formData,
      { headers: { 'Content-Type': 'multipart/form-data' } },
    );
    return res.data.data;
  },

  /** Export data for a given entity type. Returns a Blob for download. */
  exportData: async (entityType: string, format: string = 'xlsx') => {
    const response = await apiClient.get(`/data/${entityType}/export`, {
      params: { format },
      responseType: 'blob',
    });
    return response.data as Blob;
  },

  /** Download a blank import template for a given entity type. */
  getTemplate: async (entityType: string, format: string = 'xlsx') => {
    const response = await apiClient.get(`/data/${entityType}/template`, {
      params: { format },
      responseType: 'blob',
    });
    return response.data as Blob;
  },

  /** List operation logs (import/export audit trail). */
  listOperationLogs: async (params?: {
    page?: number;
    page_size?: number;
    user_id?: number;
    action?: string;
    entity_type?: string;
  }) => {
    const res = await apiClient.get<PaginatedResponse<OperationLog>>(
      '/data/logs',
      { params },
    );
    return res.data.data;
  },
};
