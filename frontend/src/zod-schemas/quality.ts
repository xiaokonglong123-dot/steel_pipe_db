/**
 * 质检模块 Zod Schema — 质量证书、钢级参考标准、附件
 *
 * 校验质量证书（含机械性能、NDT 数据）、
 * 钢级参考标准和证书附件的 API 响应结构。
 */
import { z } from 'zod';

const nullableString = z.string().optional();
const nullableNumber = z.number().optional();

export const qualityCertSchema = z.object({
  id: z.number(),
  cert_number: z.string(),
  pipe_type: z.string(),
  pipe_id: z.number(),
  cert_date: nullableString,
  result: z.string(),
  inspector: nullableString,
  inspection_body: nullableString,
  notes: nullableString,
  created_at: z.string(),
  updated_at: z.string(),
  deleted_at: nullableString,
});

export const gradeRefSchema = z.object({
  id: z.number(),
  grade: z.string(),
  yield_strength_min: nullableNumber,
  yield_strength_max: nullableNumber,
  tensile_strength_min: nullableNumber,
  hardness_max: nullableString,
  carbon_content_max: nullableNumber,
  manganese_content_max: nullableNumber,
  phosphorus_content_max: nullableNumber,
  sulfur_content_max: nullableNumber,
  notes: nullableString,
});

export const pipeAttachmentSchema = z.object({
  id: z.number(),
  pipe_type: z.string(),
  pipe_id: z.number(),
  file_name: z.string(),
  file_path: z.string(),
  file_size: nullableNumber,
  content_type: nullableString,
  uploaded_by: nullableNumber,
  created_at: z.string(),
});
