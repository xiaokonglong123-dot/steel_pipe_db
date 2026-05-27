type InventorySummaryParams = { location_id?: number; grade?: string };
type OrderReportParams = { start_date?: string; end_date?: string; order_type?: string };
type QualityReportParams = { start_date?: string; end_date?: string; grade?: string };

export const reportQueryKeys = {
  inventorySummary: (params?: InventorySummaryParams) => ['inventory-summary', params] as const,
  order: (params?: OrderReportParams) => ['order-report', params] as const,
  quality: (params?: QualityReportParams) => ['quality-report', params] as const,
  dashboard: ['dashboard'] as const,
};
