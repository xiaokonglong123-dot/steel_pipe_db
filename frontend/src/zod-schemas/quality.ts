// 质检模块 Zod Schema — 质量证书、钢级参考标准、附件
import { z } from 'zod';

export const qualityCertSchema = z.object({
  id: z.number(),
  cert_number: z.string(),
  batch_number: z.string().optional(),
  pipe_type: z.string(),
  grade: z.string(),
  od: z.number(),
  wt: z.number(),
  length: z.number().optional(),
  quantity: z.number(),
  heat_number: z.string().optional(),
  manufacturer: z.string().optional(),
  production_date: z.string().optional(),
  test_pressure: z.number().optional(),
  yield_strength: z.number().optional(),
  tensile_strength: z.number().optional(),
  elongation: z.number().optional(),
  hardness: z.number().optional(),
  inspection_standard: z.string().optional(),
  inspector: z.string().optional(),
  cert_date: z.string().optional(),
  status: z.string(),
  notes: z.string().optional(),
  created_at: z.string(),
  updated_at: z.string(),
}).strict();

export const gradeRefSchema = z.object({
  id: z.number(),
  grade: z.string(),
  od_range: z.string().optional(),
  wt_range: z.string().optional(),
  min_yield: z.number().optional(),
  max_yield: z.number().optional(),
  min_tensile: z.number().optional(),
  min_elongation: z.number().optional(),
  max_hardness: z.number().optional(),
  standard: z.string().optional(),
}).strict();

export const attachmentSchema = z.object({
  id: z.number(),
  cert_id: z.number(),
  file_name: z.string(),
  file_type: z.string(),
  file_url: z.string(),
  uploaded_at: z.string(),
}).strict();
