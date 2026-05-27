import { useQuery } from '@tanstack/react-query';
import { reportApi } from '../api/reportApi';
import { reportQueryKeys } from '../queryKeys';

export function useInventorySummary(params?: { location_id?: number; grade?: string }) {
  return useQuery({
    queryKey: reportQueryKeys.inventorySummary(params),
    queryFn: () => reportApi.getInventorySummary(params),
  });
}

export function useOrderReport(params?: { start_date?: string; end_date?: string; order_type?: string }) {
  return useQuery({
    queryKey: reportQueryKeys.order(params),
    queryFn: () => reportApi.getOrderReport(params),
  });
}

export function useQualityReport(params?: { start_date?: string; end_date?: string; grade?: string }) {
  return useQuery({
    queryKey: reportQueryKeys.quality(params),
    queryFn: () => reportApi.getQualityReport(params),
  });
}

export function useDashboard() {
  return useQuery({
    queryKey: reportQueryKeys.dashboard,
    queryFn: () => reportApi.getDashboard(),
  });
}
