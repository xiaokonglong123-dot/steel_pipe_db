export interface InventorySummary {
  pipe_type: string;
  grade: string;
  total_quantity: number;
  location: string;
}

export interface OrderReport {
  period: string;
  order_count: number;
  total_amount: number;
  by_status: Record<string, number>;
}

export interface QualityReport {
  period: string;
  total_certificates: number;
  passed: number;
  failed: number;
  by_grade: Record<string, { total: number; passed: number; failed: number }>;
}

export interface DashboardData {
  total_pipes: number;
  total_inventory: number;
  pending_orders: number;
  recent_quality_certs: number;
  inventory_by_type: { pipe_type: string; quantity: number }[];
  orders_by_status: { status: string; count: number }[];
  recent_activities: { id: number; action: string; timestamp: string; detail: string }[];
}

export interface DashboardStats {
  total_pipes: number;
  total_inventory: number;
  pending_orders: number;
  recent_quality_certs: number;
}
