import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import type { InventorySummary, OrderReport, QualityReport, DashboardData } from '../types';

export const reportApi = {
  getInventorySummary: async (params?: { location_id?: number; grade?: string }) => {
    const res = await apiClient.get<ApiResponse<InventorySummary[]>>(
      '/reports/inventory-summary',
      { params },
    );
    return res.data.data;
  },

  getOrderReport: async (params?: { start_date?: string; end_date?: string; order_type?: string }) => {
    const res = await apiClient.get<ApiResponse<OrderReport>>(
      '/reports/order-report',
      { params },
    );
    return res.data.data;
  },

  getQualityReport: async (params?: { start_date?: string; end_date?: string; grade?: string }) => {
    const res = await apiClient.get<ApiResponse<QualityReport>>(
      '/reports/quality-report',
      { params },
    );
    return res.data.data;
  },

  getDashboard: async () => {
    const res = await apiClient.get<ApiResponse<DashboardData>>('/reports/dashboard');
    return res.data.data;
  },
};
