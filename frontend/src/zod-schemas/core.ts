/**
 * 核心数据模型 Zod Schema — 钢管、筛管、客户、供应商、用户、认证
 *
 * 定义系统中最基础的业务实体结构，
 * 用于 API 响应运行时校验和 TypeScript 类型推导。
 */
import { z } from 'zod';

export const seamlessPipeSchema = z.object({
  id: z.number(),
  pipe_number: z.string(),
  batch_number: z.string().optional(),
  pipe_type: z.string(),
  grade: z.string(),
  od: z.number(),
  wt: z.number(),
  length: z.number().optional(),
  weight_per_unit: z.number().optional(),
  end_type: z.string().optional(),
  coupling_type: z.string().optional(),
  coupling_od: z.number().optional(),
  coupling_length: z.number().optional(),
  heat_number: z.string().optional(),
  serial_number: z.string().optional(),
  manufacturer: z.string().optional(),
  production_date: z.string().optional(),
  cert_number: z.string().optional(),
  location_id: z.number().optional(),
  status: z.string(),
  notes: z.string().optional(),
  created_at: z.string(),
  updated_at: z.string(),
}).strict();

export const screenPipeSchema = z.object({
  id: z.number(),
  pipe_number: z.string(),
  batch_number: z.string().optional(),
  screen_type: z.string(),
  slot_size: z.number().optional(),
  filtration_grade: z.string().optional(),
  base_od: z.number(),
  base_wt: z.number(),
  base_grade: z.string(),
  base_end_type: z.string().optional(),
  length: z.number().optional(),
  weight_per_unit: z.number().optional(),
  heat_number: z.string().optional(),
  serial_number: z.string().optional(),
  manufacturer: z.string().optional(),
  production_date: z.string().optional(),
  cert_number: z.string().optional(),
  location_id: z.number().optional(),
  status: z.string(),
  notes: z.string().optional(),
  created_at: z.string(),
  updated_at: z.string(),
}).strict();

export const customerSchema = z.object({
  id: z.number(),
  code: z.string(),
  name: z.string(),
  contact_person: z.string().optional(),
  phone: z.string().optional(),
  email: z.string().optional(),
  address: z.string().optional(),
  tax_id: z.string().optional(),
  bank_info: z.string().optional(),
  industry: z.string().optional(),
  status: z.string(),
  notes: z.string().optional(),
  created_at: z.string(),
  updated_at: z.string(),
}).strict();

export const supplierSchema = z.object({
  id: z.number(),
  code: z.string(),
  name: z.string(),
  contact_person: z.string().optional(),
  phone: z.string().optional(),
  email: z.string().optional(),
  address: z.string().optional(),
  tax_id: z.string().optional(),
  bank_info: z.string().optional(),
  grade_supply: z.string().optional(),
  status: z.string(),
  notes: z.string().optional(),
  created_at: z.string(),
  updated_at: z.string(),
}).strict();

export const userInfoSchema = z.object({
  id: z.number(),
  username: z.string(),
  display_name: z.string(),
  role: z.string(),
  email: z.string().nullable().optional(),
  phone: z.string().nullable().optional(),
}).strict();

export const loginResponseSchema = z.object({
  token: z.string(),
  user: userInfoSchema,
}).strict();

export const tokenResponseSchema = z.object({
  token: z.string(),
}).strict();
