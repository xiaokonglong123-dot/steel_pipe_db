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
    return validateResponse(res.data.data, z.array(inventorySummarySchema));
  },

  getOrderReport: async (params?: { start_date?: string; end_date?: string; order_type?: string }) => {
    const res = await apiClient.get<ApiResponse<OrderReport>>(
      '/reports/order-report',
      { params },
    );
    return validateResponse(res.data.data, orderReportSchema);
  },

  getQualityReport: async (params?: { start_date?: string; end_date?: string; grade?: string }) => {
    const res = await apiClient.get<ApiResponse<QualityReport>>(
      '/reports/quality-report',
      { params },
    );
    return validateResponse(res.data.data, qualityReportSchema);
  },

  getDashboard: async () => {
    const res = await apiClient.get<ApiResponse<DashboardData>>('/reports/dashboard');
    return validateResponse(res.data.data, dashboardDataSchema);
  },
};
