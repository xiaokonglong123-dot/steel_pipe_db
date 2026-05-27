/**
 * Global search module Zod schemas — pipe, inbound/outbound, order search results
 *
 * Validates the response shapes for different result types from the unified search,
 * covering pipes, inbound/outbound docs, purchase/sales orders.
 */
import { z } from 'zod';

export const searchPipeResultSchema = z.object({
  id: z.number(),
  pipe_number: z.string(),
  grade: z.string(),
  od: z.number(),
  wt: z.number(),
  length: z.number().optional(),
  status: z.string(),
  pipe_type: z.string(),
});

export const searchInboundResultSchema = z.object({
  id: z.number(),
  inbound_no: z.string(),
  inbound_type: z.string(),
  approval_status: z.string(),
  created_at: z.string(),
  notes: z.string().optional(),
});

export const searchOutboundResultSchema = z.object({
  id: z.number(),
  outbound_no: z.string(),
  outbound_type: z.string(),
  approval_status: z.string(),
  created_at: z.string(),
  notes: z.string().optional(),
});

export const searchPurchaseOrderResultSchema = z.object({
  id: z.number(),
  order_number: z.string(),
  supplier_name: z.string().optional(),
  status: z.string(),
  order_date: z.string(),
  total_amount: z.number().optional(),
});

export const searchSalesOrderResultSchema = z.object({
  id: z.number(),
  order_number: z.string(),
  customer_name: z.string().optional(),
  status: z.string(),
  order_date: z.string(),
  total_amount: z.number().optional(),
});
