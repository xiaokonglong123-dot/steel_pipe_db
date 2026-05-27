/**
 * 标签打印模块 Zod Schema — 钢管标签与通用标签数据
 *
 * 校验钢管标签（含规格、炉号、证书号等）
 * 和通用标签数据的 API 响应结构。
 */
import { z } from 'zod';

export const pipeLabelSchema = z.object({
  pipe_number: z.string(),
  pipe_type: z.string(),
  grade: z.string(),
  od: z.number(),
  wt: z.number(),
  length: z.number().optional(),
  heat_number: z.string().optional(),
  serial_number: z.string().optional(),
  cert_number: z.string().optional(),
  manufacturer: z.string().optional(),
  production_date: z.string().optional(),
  location: z.string().optional(),
  status: z.string(),
}).strict();

export const labelDataSchema = z.object({
  label_id: z.string(),
  content: z.string(),
  format: z.string(),
}).strict();
