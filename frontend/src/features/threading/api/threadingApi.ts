// Threading API — records + API 5CT engineering calcs
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';

export interface CalcResult {
  od: number;
  wt: number;
  grade: string;
  connection_type: string;
  joint_efficiency: number;
  burst_pressure: number;
  collapse_pressure: number;
  tension_capacity: number;
  cached: boolean;
}

export interface DesignCheckOutput {
  depth: number;
  external_pressure_psi: number;
  burst_safety_factor: number;
  collapse_safety_factor: number;
  tension_safety_factor: number;
  verdict: string;
}

const calcSchema = z.object({
  od: z.number(), wt: z.number(), grade: z.string(), connection_type: z.string(),
  joint_efficiency: z.number(), burst_pressure: z.number(), collapse_pressure: z.number(),
  tension_capacity: z.number(), cached: z.boolean(),
}).passthrough();
const singleOf = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: item }).passthrough();
const designSchema = z.object({
  depth: z.number(), external_pressure_psi: z.number(),
  burst_safety_factor: z.number(), collapse_safety_factor: z.number(),
  tension_safety_factor: z.number(), verdict: z.string(),
}).passthrough();

export const threadingApi = {
  calc: async (data: { od: number; wt: number; grade: string; connection_type: string }): Promise<CalcResult> => {
    const res = await apiClient.post<ApiResponse<CalcResult>>('/threading/calc', data);
    return validateResponse(singleOf(calcSchema), res.data).data;
  },
  designCheck: async (data: { od: number; wt: number; grade: string; connection_type: string; depth: number; fluid_density?: number }): Promise<DesignCheckOutput> => {
    const res = await apiClient.post<ApiResponse<DesignCheckOutput>>('/casing/design-check', data);
    return validateResponse(singleOf(designSchema), res.data).data;
  },
};
