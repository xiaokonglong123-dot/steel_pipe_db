// 库存模块 Zod Schema — 出入库记录、库位、盘点、钢管搜索
import { z } from 'zod';

export const inboundRecordSchema = z.object({
  id: z.number(),
  inbound_no: z.string(),
  inbound_type: z.string(),
  order_id: z.number().optional(),
  supplier_id: z.number().optional(),
  notes: z.string().optional(),
  approval_status: z.string(),
  handled_by: z.number().optional(),
  handled_at: z.string().optional(),
  created_at: z.string(),
  updated_at: z.string(),
}).strict();

export const inboundItemSchema = z.object({
  id: z.number(),
  inbound_id: z.number(),
  pipe_type: z.string(),
  pipe_id: z.number(),
  created_at: z.string(),
}).strict();

export const inboundDetailSchema = z.object({
  record: inboundRecordSchema,
  items: z.array(inboundItemSchema),
}).strict();

export const outboundRecordSchema = z.object({
  id: z.number(),
  outbound_no: z.string(),
  outbound_type: z.string(),
  order_id: z.number().optional(),
  customer_id: z.number().optional(),
  notes: z.string().optional(),
  approval_status: z.string(),
  handled_by: z.number().optional(),
  handled_at: z.string().optional(),
  created_at: z.string(),
  updated_at: z.string(),
}).strict();

export const outboundItemSchema = z.object({
  id: z.number(),
  outbound_id: z.number(),
  pipe_type: z.string(),
  pipe_id: z.number(),
  created_at: z.string(),
}).strict();

export const outboundDetailSchema = z.object({
  record: outboundRecordSchema,
  items: z.array(outboundItemSchema),
}).strict();

export const locationSchema = z.object({
  id: z.number(),
  zone_code: z.string(),
  shelf_code: z.string(),
  level_code: z.string(),
  full_code: z.string(),
  description: z.string().optional(),
  capacity: z.number().optional(),
  used_count: z.number(),
  is_active: z.boolean(),
  created_at: z.string(),
  updated_at: z.string(),
}).strict();

export const inventoryLogSchema = z.object({
  id: z.number(),
  pipe_type: z.string(),
  pipe_id: z.number(),
  change_type: z.string(),
  ref_type: z.string().optional(),
  ref_id: z.number().optional(),
  from_location_id: z.number().optional(),
  to_location_id: z.number().optional(),
  notes: z.string().optional(),
  created_by: z.number().optional(),
  created_at: z.string(),
}).strict();

export const inventoryCheckRecordSchema = z.object({
  id: z.number(),
  check_no: z.string(),
  location_id: z.number().optional(),
  status: z.string(),
  notes: z.string().optional(),
  created_by: z.number().optional(),
  created_at: z.string(),
  updated_at: z.string(),
}).strict();

export const inventoryCheckItemSchema = z.object({
  id: z.number(),
  check_id: z.number(),
  pipe_type: z.string(),
  pipe_id: z.number(),
  expected_status: z.string(),
  found_status: z.string().optional(),
  is_match: z.boolean().optional(),
  notes: z.string().optional(),
  created_at: z.string(),
}).strict();

export const checkDetailSchema = z.object({
  record: inventoryCheckRecordSchema,
  items: z.array(inventoryCheckItemSchema),
}).strict();

export const pipeSearchResultSchema = z.object({
  id: z.number(),
  pipe_type: z.string(),
  pipe_number: z.string(),
  grade: z.string(),
  od: z.number(),
  wt: z.number(),
  status: z.string(),
  location_id: z.number().optional(),
}).strict();

// Stock query result (dynamically built from seamless/screen pipes with location join)
export const stockItemSchema = z.object({
  id: z.number(),
  pipe_type: z.string(),
  pipe_number: z.string().optional(),
  grade: z.string().optional(),
  od: z.number().optional(),
  wt: z.number().optional(),
  status: z.string(),
  location_id: z.number().optional(),
  full_code: z.string().optional(),
  total_count: z.number().optional(),
}).passthrough();

// Trace pipe lifecycle result
export const tracePipeSchema = z.object({
  pipe_type: z.string(),
  pipe_number: z.string(),
  grade: z.string(),
  od: z.number(),
  wt: z.number(),
  current_status: z.string(),
  current_location_id: z.number().nullable(),
}).passthrough();

// Trace heat number result item
export const traceHeatItemSchema = z.object({
  pipe_type: z.string(),
  pipe_number: z.string(),
  grade: z.string().optional(),
  od: z.number().optional(),
  wt: z.number().optional(),
  status: z.string().optional(),
}).passthrough();

// Trace order result
export const traceOrderSchema = z.record(z.unknown());
