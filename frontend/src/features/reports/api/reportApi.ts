// 报表 API — 库存汇总/订单报表/质量报表/仪表盘数据
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import type { InventorySummary, OrderReport, QualityReport, DashboardData } from '../types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';
import {
  inventorySummarySchema,
  orderReportSchema,
  qualityReportSchema,
  dashboardDataSchema,
} from '@/zod-schemas/reports';

export const reportApi = {
  getInventorySummary: async (params?: { location_id?: number; grade?: string }) => {
    const res = await apiClient.get<ApiResponse<InventorySummary[]>>(
      '/reports/inventory-summary',
      { params },
    );
    return validateResponse(z.array(inventorySummarySchema), res.data.data);
  },

  getOrderReport: async (params?: { start_date?: string; end_date?: string; order_type?: string }) => {
    const res = await apiClient.get<ApiResponse<OrderReport>>(
      '/reports/order-report',
      { params },
    );
    return validateResponse(orderReportSchema, res.data.data);
  },

  getQualityReport: async (params?: { start_date?: string; end_date?: string; grade?: string }) => {
    const res = await apiClient.get<ApiResponse<QualityReport>>(
      '/reports/quality-report',
      { params },
    );
    return validateResponse(qualityReportSchema, res.data.data);
  },

  getDashboard: async () => {
    const res = await apiClient.get<ApiResponse<DashboardData>>('/reports/dashboard');
    return validateResponse(dashboardDataSchema, res.data.data);
  },
};
