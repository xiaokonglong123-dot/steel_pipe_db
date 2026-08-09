/**
 * Inventory module Zod schemas — inbound/outbound records, locations, stocktakes, traceability
 *
 * Validates API response shapes for inbound/outbound docs & items, location info,
 * stocktake records, stock query results, pipe traceability data, etc.
 */
import { z } from 'zod';

const nullableString = z.string().nullable().optional();
const nullableNumber = z.number().nullable().optional();

export const inboundRecordSchema = z.object({
  id: z.number(),
  inbound_no: z.string(),
  inbound_type: z.string(),
  order_id: nullableNumber,
  supplier_id: nullableNumber,
  notes: nullableString,
  approval_status: z.string(),
  rejection_reason: nullableString,
  approval_reason: nullableString,
  handled_by: nullableNumber,
  handled_at: nullableString,
  created_at: z.string(),
  updated_at: z.string(),
  deleted_at: nullableString,
}).passthrough();

export const inboundItemSchema = z.object({
  id: z.number(),
  inbound_id: z.number(),
  item_id: z.number(),
  quantity: z.number(),
  created_at: z.string(),
}).passthrough();

export const inboundDetailSchema = z.object({
  record: inboundRecordSchema,
  items: z.array(inboundItemSchema),
}).passthrough();

export const outboundRecordSchema = z.object({
  id: z.number(),
  outbound_no: z.string(),
  outbound_type: z.string(),
  order_id: nullableNumber,
  customer_id: nullableNumber,
  notes: nullableString,
  approval_status: z.string(),
  rejection_reason: nullableString,
  approval_reason: nullableString,
  handled_by: nullableNumber,
  handled_at: nullableString,
  created_at: z.string(),
  updated_at: z.string(),
  deleted_at: nullableString,
}).passthrough();

export const outboundItemSchema = z.object({
  id: z.number(),
  outbound_id: z.number(),
  item_id: z.number(),
  quantity: z.number(),
  created_at: z.string(),
}).passthrough();

export const outboundDetailSchema = z.object({
  record: outboundRecordSchema,
  items: z.array(outboundItemSchema),
}).passthrough();

export const locationSchema = z.object({
  id: z.number(),
  zone_code: z.string(),
  shelf_code: z.string(),
  level_code: z.string(),
  full_code: z.string(),
  description: nullableString,
  capacity: nullableNumber,
  used_count: z.number(),
  is_active: z.boolean(),
  created_at: z.string(),
  updated_at: z.string(),
  deleted_at: nullableString,
}).passthrough();

export const inventoryLogSchema = z.object({
  id: z.number(),
  pipe_type: z.string(),
  pipe_id: z.number(),
  change_type: z.string(),
  ref_type: nullableString,
  ref_id: nullableNumber,
  from_location_id: nullableNumber,
  to_location_id: nullableNumber,
  notes: nullableString,
  created_by: nullableNumber,
  created_at: z.string(),
}).passthrough();

export const inventoryCheckRecordSchema = z.object({
  id: z.number(),
  check_no: z.string(),
  location_id: nullableNumber,
  status: z.string(),
  notes: nullableString,
  created_by: nullableNumber,
  created_at: z.string(),
  updated_at: z.string(),
  deleted_at: nullableString,
}).passthrough();

export const inventoryCheckItemSchema = z.object({
  id: z.number(),
  check_id: z.number(),
  pipe_type: z.string(),
  pipe_id: z.number(),
  expected_status: z.string(),
  found_status: nullableString,
  is_match: z.boolean().nullable().optional(),
  notes: nullableString,
  created_at: z.string(),
}).passthrough();

export const checkDetailSchema = z.object({
  record: inventoryCheckRecordSchema,
  items: z.array(inventoryCheckItemSchema),
}).passthrough();

// Stock query result (dynamically built from seamless/screen pipes with location join)
export const stockItemSchema = z.object({
  id: z.number(),
  pipe_type: z.string(),
  pipe_number: nullableString,
  grade: nullableString,
  od: nullableNumber,
  wt: nullableNumber,
  status: z.string(),
  location_id: nullableNumber,
  full_code: nullableString,
  total_count: nullableNumber,
}).passthrough();

// Trace pipe lifecycle result
export const tracePipeSchema = z.object({
  pipe_type: z.string(),
  pipe_number: z.string(),
  grade: z.string(),
  od: z.number(),
  wt: z.number(),
  current_status: z.string(),
  current_location_id: nullableNumber,
}).passthrough();

// Trace heat number result item
export const traceHeatItemSchema = z.object({
  pipe_type: z.string(),
  pipe_number: z.string(),
  grade: nullableString,
  od: nullableNumber,
  wt: nullableNumber,
  status: nullableString,
}).passthrough();

// Trace order result
export const traceOrderSchema = z.record(z.unknown());
