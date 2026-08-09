/**
 * Order & contract module Zod schemas — purchase/sales order line items, contract terms & payments
 *
 * Validates API response shapes for purchase orders (with line items),
 * sales orders (with line items), and contracts (with terms & payment records).
 *
 * NOTE: the backend order/contract line items were generalized from pipe-specific
 * columns (`pipe_type/grade/od/wt`) to a generic `item_id` reference. The schemas
 * below match the backend models; `.passthrough()` tolerates legacy rows that
 * still carry the old pipe fields. The remaining pipe-specific UI columns in the
 * list/form pages are tracked for cleanup in phase 1 (see docs/refactor-issue-list-2026-08-09.md).
 */
import { z } from 'zod';

export const purchaseOrderItemSchema = z.object({
  id: z.number(),
  order_id: z.number(),
  item_id: z.number(),
  quantity: z.number(),
  received_quantity: z.number(),
  unit_price: z.number().optional(),
  total_price: z.number().optional(),
  notes: z.string().optional(),
  created_at: z.string(),
}).passthrough();

export const purchaseOrderSchema = z.object({
  id: z.number(),
  order_no: z.string(),
  supplier_id: z.number(),
  order_date: z.string(),
  status: z.string(),
  total_amount: z.number().optional(),
  notes: z.string().optional(),
  created_by: z.number().optional(),
  created_at: z.string(),
  updated_at: z.string(),
  deleted_at: z.string().optional(),
});

/** Purchase order detail response — matches backend PurchaseOrderDetailResponse. */
export const purchaseOrderDetailSchema = z.object({
  order: purchaseOrderSchema,
  items: z.array(purchaseOrderItemSchema),
}).passthrough();

export const salesOrderItemSchema = z.object({
  id: z.number(),
  order_id: z.number(),
  item_id: z.number(),
  quantity: z.number(),
  delivered_quantity: z.number(),
  unit_price: z.number().optional(),
  total_price: z.number().optional(),
  notes: z.string().optional(),
  created_at: z.string(),
}).passthrough();

export const salesOrderSchema = z.object({
  id: z.number(),
  order_no: z.string(),
  customer_id: z.number(),
  order_date: z.string(),
  status: z.string(),
  total_amount: z.number().optional(),
  notes: z.string().optional(),
  created_by: z.number().optional(),
  created_at: z.string(),
  updated_at: z.string(),
  deleted_at: z.string().optional(),
});

/** Sales order detail response — matches backend SalesOrderDetailResponse. */
export const salesOrderDetailSchema = z.object({
  order: salesOrderSchema,
  items: z.array(salesOrderItemSchema),
}).passthrough();

export const contractItemSchema = z.object({
  id: z.number(),
  contract_id: z.number(),
  item_id: z.number(),
  quantity: z.number(),
  unit_price: z.number().optional(),
  total_price: z.number().optional(),
  notes: z.string().optional(),
  created_at: z.string(),
}).passthrough();

export const contractPaymentSchema = z.object({
  id: z.number(),
  contract_id: z.number(),
  due_date: z.string(),
  amount: z.number(),
  payment_type: z.string(),
  is_paid: z.boolean(),
  paid_date: z.string().optional(),
  notes: z.string().optional(),
  created_at: z.string(),
});

export const contractSchema = z.object({
  id: z.number(),
  contract_no: z.string(),
  contract_type: z.enum(['purchase', 'sales']),
  title: z.string(),
  party_a: z.string(),
  party_b: z.string(),
  sign_date: z.string().optional(),
  start_date: z.string().optional(),
  end_date: z.string().optional(),
  total_amount: z.number().optional(),
  status: z.enum(['draft', 'active', 'completed', 'terminated', 'cancelled']),
  notes: z.string().optional(),
  created_by: z.number().optional(),
  created_at: z.string(),
  updated_at: z.string(),
  deleted_at: z.string().optional(),
  items: z.array(contractItemSchema).optional(),
  payments: z.array(contractPaymentSchema).optional(),
});

/** Contract detail response — matches backend ContractDetailResponse. */
export const contractDetailSchema = z.object({
  contract: contractSchema,
  items: z.array(contractItemSchema).optional(),
  payments: z.array(contractPaymentSchema).optional(),
}).passthrough();
