import client from '../../../api/client';
import type { ApiResponse } from '../../../shared/types';

export type ImportStrategy = 'skip' | 'overwrite' | 'auto_number';

export interface ImportPreviewRow {
  pipe_number: string;
  grade: string;
  od: number;
  wt: number;
  length: number;
  weight: number;
  [key: string]: unknown;
}

export interface ImportPreview {
  total_rows: number;
  preview_rows: ImportPreviewRow[];
  columns: string[];
}

export interface ImportError {
  row: number;
  reason: string;
}

export interface ImportResult {
  total_rows: number;
  success_rows: number;
  failed_rows: number;
  errors: ImportError[];
}

export type ExportType = 'inventory' | 'inbound' | 'outbound' | 'pipes';
export type ExportFormat = 'xlsx' | 'csv';

export interface ExportFilter {
  date_from?: string;
  date_to?: string;
  grade?: string;
  pipe_type?: string;
  status?: string;
  fields?: string[];
  format?: ExportFormat;
}

export const exportFieldOptions: Record<ExportType, { label: string; value: string }[]> = {
  inventory: [
    { label: '管材编号', value: 'pipe_number' },
    { label: '类型', value: 'pipe_type' },
    { label: '钢级', value: 'grade' },
    { label: '外径 (mm)', value: 'od' },
    { label: '壁厚 (mm)', value: 'wt' },
    { label: '长度 (m)', value: 'length' },
    { label: '重量 (kg)', value: 'weight' },
    { label: '状态', value: 'status' },
    { label: '库位', value: 'location' },
    { label: '入库日期', value: 'inbound_date' },
  ],
  inbound: [
    { label: '入库单号', value: 'inbound_no' },
    { label: '类型', value: 'inbound_type' },
    { label: '管材编号', value: 'pipe_number' },
    { label: '供应商', value: 'supplier' },
    { label: '数量', value: 'quantity' },
    { label: '操作人', value: 'operator' },
    { label: '入库日期', value: 'inbound_date' },
    { label: '备注', value: 'notes' },
  ],
  outbound: [
    { label: '出库单号', value: 'outbound_no' },
    { label: '类型', value: 'outbound_type' },
    { label: '管材编号', value: 'pipe_number' },
    { label: '客户', value: 'customer' },
    { label: '数量', value: 'quantity' },
    { label: '操作人', value: 'operator' },
    { label: '出库日期', value: 'outbound_date' },
    { label: '备注', value: 'notes' },
  ],
  pipes: [
    { label: '管材编号', value: 'pipe_number' },
    { label: '类型', value: 'pipe_type' },
    { label: '钢级', value: 'grade' },
    { label: '外径 (mm)', value: 'od' },
    { label: '壁厚 (mm)', value: 'wt' },
    { label: '长度 (m)', value: 'length' },
    { label: '重量 (kg)', value: 'weight' },
    { label: '接箍类型', value: 'connection_type' },
    { label: '炉批号', value: 'heat_number' },
    { label: '生产日期', value: 'production_date' },
    { label: '状态', value: 'status' },
    { label: '库位', value: 'location' },
    { label: '备注', value: 'notes' },
    { label: '创建时间', value: 'created_at' },
  ],
};

export const dataIoApi = {
  importSeamlessPipes: (file: File, strategy: ImportStrategy) => {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('strategy', strategy);
    return client.post<ApiResponse<ImportResult>>('/import/seamless-pipes', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
  },

  importScreenPipes: (file: File, strategy: ImportStrategy) => {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('strategy', strategy);
    return client.post<ApiResponse<ImportResult>>('/import/screen-pipes', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
  },

  downloadSeamlessTemplate: () =>
    client.get('/import/template/seamless-pipes', {
      responseType: 'blob',
    }),

  downloadScreenTemplate: () =>
    client.get('/import/template/screen-pipes', {
      responseType: 'blob',
    }),

  exportInventory: (filter: ExportFilter) =>
    client.post('/export/inventory', filter, {
      responseType: 'blob',
    }),

  exportInbound: (filter: ExportFilter) =>
    client.post('/export/inbound', filter, {
      responseType: 'blob',
    }),

  exportOutbound: (filter: ExportFilter) =>
    client.post('/export/outbound', filter, {
      responseType: 'blob',
    }),

  exportPipes: (filter: ExportFilter & { pipe_type?: 'seamless' | 'screen' }) =>
    client.post('/export/pipes', filter, {
      responseType: 'blob',
    }),
};
