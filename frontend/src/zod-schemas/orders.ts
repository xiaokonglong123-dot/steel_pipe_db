import { z } from 'zod';

export const purchaseOrderItemSchema = z.object({
  id: z.number(),
  purchase_order_id: z.number(),
  pipe_type: z.string(),
  grade: z.string(),
  od: z.number(),
  wt: z.number(),
  length: z.number().optional(),
  quantity: z.number(),
  unit_price: z.number(),
  total_price: z.number(),
  notes: z.string().optional(),
}).strict();

export const purchaseOrderSchema = z.object({
  id: z.number(),
  order_number: z.string(),
  supplier_id: z.number(),
  supplier_name: z.string(),
  order_date: z.string(),
  expected_date: z.string().optional(),
  status: z.string(),
  total_amount: z.number(),
  notes: z.string().optional(),
  items: z.array(purchaseOrderItemSchema),
  created_at: z.string(),
  updated_at: z.string(),
}).strict();

export const salesOrderItemSchema = z.object({
  id: z.number(),
  sales_order_id: z.number(),
  pipe_id: z.number(),
  pipe_number: z.string().optional(),
  pipe_type: z.string().optional(),
  grade: z.string().optional(),
  od: z.number().optional(),
  wt: z.number().optional(),
  length: z.number().optional(),
  quantity: z.number(),
  unit_price: z.number(),
  total_price: z.number(),
  notes: z.string().optional(),
}).strict();

export const salesOrderSchema = z.object({
  id: z.number(),
  order_number: z.string(),
  customer_id: z.number(),
  customer_name: z.string().optional(),
  order_date: z.string(),
  expected_delivery: z.string().optional(),
  status: z.string(),
  total_amount: z.number(),
  notes: z.string().optional(),
  items: z.array(salesOrderItemSchema),
  created_at: z.string(),
  updated_at: z.string(),
}).strict();

export const contractItemSchema = z.object({
  id: z.number(),
  contract_id: z.number(),
  pipe_type: z.enum(['seamless', 'screen']),
  grade: z.string(),
  od: z.number(),
  wt: z.number(),
  length: z.number().optional(),
  quantity: z.number(),
  unit_price: z.number(),
  total_price: z.number(),
  delivery_date: z.string().optional(),
  notes: z.string().optional(),
}).strict();

export const contractPaymentSchema = z.object({
  id: z.number(),
  contract_id: z.number(),
  payment_date: z.string(),
  amount: z.number(),
  payment_method: z.string().optional(),
  reference_number: z.string().optional(),
  notes: z.string().optional(),
}).strict();

export const contractSchema = z.object({
  id: z.number(),
  contract_number: z.string(),
  contract_name: z.string(),
  contract_type: z.enum(['purchase', 'sales']),
  party_a: z.string(),
  party_b: z.string(),
  sign_date: z.string().optional(),
  start_date: z.string().optional(),
  end_date: z.string().optional(),
  total_amount: z.number(),
  paid_amount: z.number(),
  status: z.enum(['draft', 'active', 'completed', 'terminated']),
  notes: z.string().optional(),
  created_at: z.string(),
  updated_at: z.string(),
  items: z.array(contractItemSchema).optional(),
  payments: z.array(contractPaymentSchema).optional(),
}).strict();
