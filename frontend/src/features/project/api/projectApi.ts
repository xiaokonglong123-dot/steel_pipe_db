// Project API — projects, WBS, financials
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';

export interface Project {
  id: number;
  project_no: string;
  name: string;
  status: string;
  budget: string;
}

export interface WbsElement {
  id: number;
  code: string;
  name: string;
  parent_id: number | null;
  progress_pct: string;
}

export interface ProjectFinancials {
  budget: string;
  expense_total: string;
  revenue_total: string;
  remaining: string;
}

const projectSchema = z.object({
  id: z.number(), project_no: z.string(), name: z.string(), status: z.string(), budget: z.string(),
}).passthrough();
const wbsSchema = z.object({
  id: z.number(), code: z.string(), name: z.string(), parent_id: z.number().nullable(), progress_pct: z.string(),
}).passthrough();
const finSchema = z.object({
  budget: z.string(), expense_total: z.string(), revenue_total: z.string(), remaining: z.string(),
}).passthrough();
const arrayOf = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: z.array(item) }).passthrough();
const singleOf = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: item }).passthrough();

export const projectApi = {
  listProjects: async () => {
    const res = await apiClient.get<ApiResponse<Project[]>>('/projects');
    return validateResponse(arrayOf(projectSchema), res.data).data;
  },
  createProject: async (data: { name: string; budget?: number; description?: string }) => {
    const res = await apiClient.post<ApiResponse<Project>>('/projects', data);
    return validateResponse(projectSchema, res.data).data;
  },
  updateStatus: async (id: number, status: string) => {
    const res = await apiClient.put<ApiResponse<Project>>(`/projects/${id}`, { status });
    return validateResponse(projectSchema, res.data).data;
  },
  wbsTree: async (projectId: number) => {
    const res = await apiClient.get<ApiResponse<WbsElement[]>>(`/projects/${projectId}/wbs`);
    return validateResponse(arrayOf(wbsSchema), res.data).data;
  },
  createWbs: async (projectId: number, data: { code: string; name: string; weight_pct?: number }) => {
    const res = await apiClient.post<ApiResponse<WbsElement>>(`/projects/${projectId}/wbs`, data);
    return validateResponse(wbsSchema, res.data).data;
  },
  financials: async (projectId: number): Promise<ProjectFinancials> => {
    const res = await apiClient.get<ApiResponse<ProjectFinancials>>(`/projects/${projectId}/financials`);
    return validateResponse(singleOf(finSchema), res.data).data;
  },
};
