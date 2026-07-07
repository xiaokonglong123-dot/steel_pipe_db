// 报表模块 TypeScript 类型 — 与后端 report_service.rs 的实际输出严格对齐

// ── Inventory Summary ────────────────────────────────────────────────────────

export interface InventoryByStatus {
  status: string;
  count: number;
}

export interface InventoryByGrade {
  grade: string;
  count: number;
  pipe_type: string;
}

export interface InventoryByType {
  pipe_type: string;
  count: number;
}

export interface LocationOccupancy {
  location: string;
  max_capacity: number;
  current_usage: number;
  available: number;
  occupancy_pct: string;
}

export interface InventorySummary {
  by_status: InventoryByStatus[];
  by_grade: InventoryByGrade[];
  by_type: InventoryByType[];
  location_occupancy: LocationOccupancy[];
}

// ── Order Report ─────────────────────────────────────────────────────────────

export interface OrderReportOrder {
  period: string;
  order_count: number;
  total_amount: number;
}

export interface StatusDistribution {
  status: string;
  count: number;
}

export interface TopCustomer {
  customer: string;
  order_count: number;
  total_amount: number;
}

export interface TopSupplier {
  supplier: string;
  order_count: number;
  total_amount: number;
}

export interface OrderReport {
  type: string;
  period: string;
  orders: OrderReportOrder[];
  status_distribution: StatusDistribution[];
  top_customers?: TopCustomer[];
  top_suppliers?: TopSupplier[];
}

// ── Quality Report ───────────────────────────────────────────────────────────

export interface QualityByGrade {
  grade: string;
  pipe_type: string;
  pass_count: number;
  fail_count: number;
  total: number;
  pass_rate: string;
}

export interface QualityByMonth {
  month: string;
  total: number;
  passed: number;
  failed: number;
  pass_rate: string;
}

export interface QualityReport {
  by_grade: QualityByGrade[];
  by_month: QualityByMonth[];
}

// ── Dashboard ────────────────────────────────────────────────────────────────

export interface DashboardRecentInbound {
  record_no: string;
  type: string;
  approval_status: string;
  created_at: string;
}

export interface DashboardRecentOutbound {
  record_no: string;
  type: string;
  approval_status: string;
  created_at: string;
}

export interface DashboardPendingApproval {
  id: number;
  reference_no: string;
  reference_type: string;
}

export interface DashboardRecentQualityFailure {
  cert_no: string;
  pipe_type: string;
  pipe_id: string;
  inspect_date: string;
  notes: string;
}

export interface DashboardData {
  total_stock: number;
  inbound_30d: number;
  outbound_30d: number;
  recent_inbound: DashboardRecentInbound[];
  recent_outbound: DashboardRecentOutbound[];
  pending_approvals: number;
  pending_approval_list: DashboardPendingApproval[];
  recent_quality_failures: DashboardRecentQualityFailure[];
}

export interface DashboardStats {
  total_stock: number;
  inbound_30d: number;
  outbound_30d: number;
  pending_approvals: number;
}
