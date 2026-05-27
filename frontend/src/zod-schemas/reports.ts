/**
 * 报表模块 Zod Schema — 库存汇总、订单报表、质量报表、仪表盘
 *
 * 校验库存汇总、订单统计、质量分析报告、
 * 仪表盘概览和首页统计数据等 API 响应。
 */
import { z } from 'zod';

export const inventorySummarySchema = z.object({
  pipe_type: z.string(),
  grade: z.string(),
  total_quantity: z.number(),
  location: z.string(),
}).strict();

export const orderReportSchema = z.object({
  period: z.string(),
  order_count: z.number(),
  total_amount: z.number(),
  by_status: z.record(z.string(), z.number()),
}).strict();

export const qualityReportSchema = z.object({
  period: z.string(),
  total_certificates: z.number(),
  passed: z.number(),
  failed: z.number(),
  by_grade: z.record(
    z.string(),
    z.object({
      total: z.number(),
      passed: z.number(),
      failed: z.number(),
    }).strict(),
  ),
}).strict();

export const dashboardDataSchema = z.object({
  total_pipes: z.number(),
  total_inventory: z.number(),
  pending_orders: z.number(),
  recent_quality_certs: z.number(),
  inventory_by_type: z.array(
    z.object({
      pipe_type: z.string(),
      quantity: z.number(),
    }).strict(),
  ),
  orders_by_status: z.array(
    z.object({
      status: z.string(),
      count: z.number(),
    }).strict(),
  ),
  recent_activities: z.array(
    z.object({
      id: z.number(),
      action: z.string(),
      timestamp: z.string(),
      detail: z.string(),
    }).strict(),
  ),
}).strict();

export const dashboardStatsSchema = z.object({
  total_pipes: z.number(),
  total_inventory: z.number(),
  pending_orders: z.number(),
  recent_quality_certs: z.number(),
}).strict();
