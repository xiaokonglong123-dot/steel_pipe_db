import { useQuery } from '@tanstack/react-query';
import { reportApi } from '../api/reportApi';

export function useInventorySummary(params?: { location_id?: number; grade?: string }) {
  return useQuery({
    queryKey: ['inventory-summary', params],
    queryFn: () => reportApi.getInventorySummary(params),
  });
}

export function useOrderReport(params?: { start_date?: string; end_date?: string; order_type?: string }) {
  return useQuery({
    queryKey: ['order-report', params],
    queryFn: () => reportApi.getOrderReport(params),
  });
}

export function useQualityReport(params?: { start_date?: string; end_date?: string; grade?: string }) {
  return useQuery({
    queryKey: ['quality-report', params],
    queryFn: () => reportApi.getQualityReport(params),
  });
}

export function useDashboard() {
  return useQuery({
    queryKey: ['dashboard'],
    queryFn: () => reportApi.getDashboard(),
  });
}
