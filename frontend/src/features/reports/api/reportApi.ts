import client from '../../../api/client';
import { ApiListResponse } from '../../../shared/types';

export interface InventoryReport {
  pipe_type: string;
  grade: string;
  total: number;
  in_stock: number;
  outbound: number;
  scrapped: number;
}

export interface QualityReport {
  month: string;
  total_inspected: number;
  pass_count: number;
  fail_rate: number;
}

export interface OrderReport {
  month: string;
  po_count: number;
  so_count: number;
  po_amount: number;
  so_amount: number;
}

export interface ReportFilter {
  date_from?: string;
  date_to?: string;
  report_type?: string;
}

export const reportApi = {
  getStockSummary: (params?: ReportFilter) =>
    client.get<ApiListResponse<InventoryReport>>('/reports/stock-summary', { params }),

  getStockByGrade: (params?: ReportFilter) =>
    client.get<ApiListResponse<InventoryReport>>('/reports/stock-by-grade', { params }),

  getStockByLocation: (params?: ReportFilter) =>
    client.get<ApiListResponse<InventoryReport>>('/reports/stock-by-location', { params }),

  getInboundSummary: (params?: ReportFilter) =>
    client.get<ApiListResponse<InventoryReport>>('/reports/inbound-summary', { params }),

  getOutboundSummary: (params?: ReportFilter) =>
    client.get<ApiListResponse<InventoryReport>>('/reports/outbound-summary', { params }),

  getMonthlyFlow: (params?: ReportFilter) =>
    client.get<ApiListResponse<OrderReport>>('/reports/monthly-flow', { params }),

  getPurchaseSummary: (params?: ReportFilter) =>
    client.get<ApiListResponse<OrderReport>>('/reports/purchase-summary', { params }),

  getSalesSummary: (params?: ReportFilter) =>
    client.get<ApiListResponse<OrderReport>>('/reports/sales-summary', { params }),
};
