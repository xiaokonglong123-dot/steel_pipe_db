/**
 * 核心数据模型 Zod Schema — 钢管、筛管、客户、供应商、用户、认证
 *
 * 定义系统中最基础的业务实体结构，
 * 用于 API 响应运行时校验和 TypeScript 类型推导。
 */
import { z } from 'zod';

const nullableString = z.string().optional();
const nullableNumber = z.number().optional();

export const seamlessPipeSchema = z.object({
  id: z.number(),
  pipe_number: z.string(),
  batch_number: nullableString,
  pipe_type: z.string(),
  grade: z.string(),
  od: z.number(),
  wt: z.number(),
  length: nullableNumber,
  weight_per_unit: nullableNumber,
  end_type: nullableString,
  coupling_type: nullableString,
  coupling_od: nullableNumber,
  coupling_length: nullableNumber,
  heat_number: nullableString,
  serial_number: nullableString,
  manufacturer: nullableString,
  production_date: nullableString,
  cert_number: nullableString,
  location_id: nullableNumber,
  status: z.string(),
  notes: nullableString,
  created_at: z.string(),
  updated_at: z.string(),
  deleted_at: nullableString,
}).passthrough();

export const screenPipeSchema = z.object({
  id: z.number(),
  pipe_number: z.string(),
  batch_number: nullableString,
  screen_type: z.string(),
  slot_size: nullableNumber,
  filtration_grade: nullableString,
  base_od: z.number(),
  base_wt: z.number(),
  base_grade: z.string(),
  base_end_type: nullableString,
  length: nullableNumber,
  weight_per_unit: nullableNumber,
  heat_number: nullableString,
  serial_number: nullableString,
  manufacturer: nullableString,
  production_date: nullableString,
  cert_number: nullableString,
  location_id: nullableNumber,
  status: z.string(),
  notes: nullableString,
  created_at: z.string(),
  updated_at: z.string(),
  deleted_at: nullableString,
}).passthrough();

export const customerSchema = z.object({
  id: z.number(),
  customer_code: z.string(),
  name: z.string(),
  contact_person: nullableString,
  phone: nullableString,
  email: nullableString,
  address: nullableString,
  is_active: z.boolean(),
  notes: nullableString,
  created_at: z.string(),
  updated_at: z.string(),
  deleted_at: nullableString,
}).passthrough();

export const supplierSchema = z.object({
  id: z.number(),
  supplier_code: z.string(),
  name: z.string(),
  contact_person: nullableString,
  phone: nullableString,
  email: nullableString,
  address: nullableString,
  is_active: z.boolean(),
  notes: nullableString,
  created_at: z.string(),
  updated_at: z.string(),
  deleted_at: nullableString,
}).passthrough();

export const userInfoSchema = z.object({
  id: z.number(),
  username: z.string(),
  display_name: z.string(),
  role: z.string(),
  email: z.string().nullable().optional(),
  phone: z.string().nullable().optional(),
}).passthrough();

export const loginResponseSchema = z.object({
  token: z.string(),
  refresh_token: z.string().optional(),
  user: userInfoSchema,
}).passthrough();

export const tokenResponseSchema = z.object({
  token: z.string(),
  refresh_token: z.string().optional(),
}).passthrough();
