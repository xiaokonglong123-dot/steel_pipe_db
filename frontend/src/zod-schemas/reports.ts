/**
 * 报表模块 Zod Schema — 库存汇总、订单报表、质量报表、仪表盘
 *
 * 校验库存汇总、订单统计、质量分析报告、
 * 仪表盘概览和首页统计数据等 API 响应。
 *
 * 这些 schema 验证 `res.data`（后端 ApiResponse 的内层 data 对象），
 * 因此描述的是嵌套 JSON 对象本身，而非外层数组。
 *
 * 后端实际返回形状见 backend/src/services/report_service.rs
 */
import { z } from 'zod';

// ── Inventory Summary ────────────────────────────────────────────────────────
// Backend returns: { by_status, by_grade, by_type, location_occupancy }

const byStatusItemSchema = z.object({
  status: z.string(),
  count: z.number(),
}).passthrough();

const byGradeItemSchema = z.object({
  grade: z.string(),
  count: z.number(),
  pipe_type: z.string(),
}).passthrough();

const byTypeItemSchema = z.object({
  pipe_type: z.string(),
  count: z.number(),
}).passthrough();

const locationOccupancyItemSchema = z.object({
  location: z.string(),
  max_capacity: z.number(),
  current_usage: z.number(),
  available: z.number(),
  occupancy_pct: z.string(),
}).passthrough();

export const inventorySummarySchema = z.object({
  by_status: z.array(byStatusItemSchema),
  by_grade: z.array(byGradeItemSchema),
  by_type: z.array(byTypeItemSchema),
  location_occupancy: z.array(locationOccupancyItemSchema),
}).passthrough();

// ── Order Report ─────────────────────────────────────────────────────────────
// Backend returns: { type, period, orders, status_distribution, top_customers | top_suppliers }

const orderItemSchema = z.object({
  period: z.string(),
  order_count: z.number(),
  total_amount: z.number(),
}).passthrough();

const statusDistItemSchema = z.object({
  status: z.string(),
  count: z.number(),
}).passthrough();

const topCustomerItemSchema = z.object({
  customer: z.string(),
  order_count: z.number(),
  total_amount: z.number(),
}).passthrough();

const topSupplierItemSchema = z.object({
  supplier: z.string(),
  order_count: z.number(),
  total_amount: z.number(),
}).passthrough();

export const orderReportSchema = z.object({
  type: z.string(),
  period: z.string(),
  orders: z.array(orderItemSchema),
  status_distribution: z.array(statusDistItemSchema),
  top_customers: z.array(topCustomerItemSchema).optional(),
  top_suppliers: z.array(topSupplierItemSchema).optional(),
}).passthrough();

// ── Quality Report ───────────────────────────────────────────────────────────
// Backend returns: { by_grade, by_month }

const qualityGradeItemSchema = z.object({
  grade: z.string(),
  pipe_type: z.string(),
  pass_count: z.number(),
  fail_count: z.number(),
  total: z.number(),
  pass_rate: z.string(),
}).passthrough();

const qualityMonthItemSchema = z.object({
  month: z.string(),
  total: z.number(),
  passed: z.number(),
  failed: z.number(),
  pass_rate: z.string(),
}).passthrough();

export const qualityReportSchema = z.object({
  by_grade: z.array(qualityGradeItemSchema),
  by_month: z.array(qualityMonthItemSchema),
}).passthrough();

// ── Dashboard ────────────────────────────────────────────────────────────────
// Backend returns: { total_stock, inbound_30d, outbound_30d, recent_inbound,
//                    recent_outbound, pending_approvals, pending_approval_list,
//                    recent_quality_failures }

const recentInboundItemSchema = z.object({
  record_no: z.string(),
  type: z.string(),
  approval_status: z.string(),
  created_at: z.string(),
}).passthrough();

const recentOutboundItemSchema = z.object({
  record_no: z.string(),
  type: z.string(),
  approval_status: z.string(),
  created_at: z.string(),
}).passthrough();

const pendingApprovalItemSchema = z.object({
  id: z.number(),
  reference_no: z.string(),
  reference_type: z.string(),
}).passthrough();

const recentQualityFailureItemSchema = z.object({
  cert_no: z.string(),
  pipe_type: z.string(),
  pipe_id: z.string(),
  inspect_date: z.string(),
  notes: z.string(),
}).passthrough();

export const dashboardDataSchema = z.object({
  total_stock: z.number(),
  inbound_30d: z.number(),
  outbound_30d: z.number(),
  recent_inbound: z.array(recentInboundItemSchema),
  recent_outbound: z.array(recentOutboundItemSchema),
  pending_approvals: z.number(),
  pending_approval_list: z.array(pendingApprovalItemSchema),
  recent_quality_failures: z.array(recentQualityFailureItemSchema),
}).passthrough();

export const dashboardStatsSchema = z.object({
  total_stock: z.number(),
  inbound_30d: z.number(),
  outbound_30d: z.number(),
  pending_approvals: z.number(),
}).passthrough();
