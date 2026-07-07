// 报表 API — 库存汇总/订单报表/质量报表/仪表盘数据
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import type { InventorySummary, OrderReport, QualityReport, DashboardData } from '../types';
import { validateResponse } from '@/lib/validateResponse';
import {
  inventorySummarySchema,
  orderReportSchema,
  qualityReportSchema,
  dashboardDataSchema,
} from '@/zod-schemas/reports';

export const reportApi = {
  getInventorySummary: async (params?: { location_id?: number; grade?: string }) => {
    const res = await apiClient.get<ApiResponse<InventorySummary>>(
      '/reports/inventory-summary',
      params,  // [#4] pass params directly, not wrapped in { params }
    );
    return validateResponse(inventorySummarySchema, res.data);
  },

  // [#6] backend expects `type` and `period` query params, not `order_type`
  getOrderReport: async (params?: { type?: string; period?: string }) => {
    const res = await apiClient.get<ApiResponse<OrderReport>>(
      '/reports/order-report',
      params,  // [#4] pass params directly, not wrapped in { params }
    );
    return validateResponse(orderReportSchema, res.data);
  },

  getQualityReport: async (params?: { start_date?: string; end_date?: string; grade?: string }) => {
    const res = await apiClient.get<ApiResponse<QualityReport>>(
      '/reports/quality-report',
      params,  // [#4] pass params directly, not wrapped in { params }
    );
    return validateResponse(qualityReportSchema, res.data);
  },

  getDashboard: async () => {
    const res = await apiClient.get<ApiResponse<DashboardData>>('/reports/dashboard');
    return validateResponse(dashboardDataSchema, res.data);
  },
};
