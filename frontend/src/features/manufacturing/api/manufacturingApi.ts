// Manufacturing API — BOMs, work orders, inspections, NCRs
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';

export interface Bom {
  id: number;
  name: string;
  product_type: string;
  version: number;
}

export interface WorkOrder {
  id: number;
  wo_no: string;
  product_type: string;
  quantity: string;
  status: string;
  current_step: number;
}

export interface Ncr {
  id: number;
  ncr_no: string;
  description: string;
  severity: string;
  status: string;
}

const bomSchema = z.object({ id: z.number(), name: z.string(), product_type: z.string(), version: z.number() }).passthrough();
const woSchema = z.object({
  id: z.number(), wo_no: z.string(), product_type: z.string(), quantity: z.string(),
  status: z.string(), current_step: z.number(),
}).passthrough();
const ncrSchema = z.object({
  id: z.number(), ncr_no: z.string(), description: z.string(), severity: z.string(), status: z.string(),
}).passthrough();
const arrayOf = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: z.array(item) }).passthrough();

export const manufacturingApi = {
  listBoms: async () => {
    const res = await apiClient.get<ApiResponse<Bom[]>>('/manufacturing/boms');
    return validateResponse(arrayOf(bomSchema), res.data).data;
  },
  createBom: async (data: { name: string; product_type: string; items: { material: string; quantity: number }[] }) => {
    const res = await apiClient.post<ApiResponse<Bom>>('/manufacturing/boms', data);
    return validateResponse(bomSchema, res.data).data;
  },
  listWorkOrders: async () => {
    const res = await apiClient.get<ApiResponse<WorkOrder[]>>('/manufacturing/work-orders');
    return validateResponse(arrayOf(woSchema), res.data).data;
  },
  createWorkOrder: async (data: { bom_id?: number; product_type: string; quantity: number }) => {
    const res = await apiClient.post<ApiResponse<WorkOrder>>('/manufacturing/work-orders', data);
    return validateResponse(woSchema, res.data).data;
  },
  startWorkOrder: async (id: number) => {
    const res = await apiClient.post<ApiResponse<WorkOrder>>(`/manufacturing/work-orders/${id}/start`);
    return validateResponse(woSchema, res.data).data;
  },
  completeStep: async (id: number) => {
    const res = await apiClient.post<ApiResponse<WorkOrder>>(`/manufacturing/work-orders/${id}/complete-step`);
    return validateResponse(woSchema, res.data).data;
  },
  listNcrs: async () => {
    const res = await apiClient.get<ApiResponse<Ncr[]>>('/manufacturing/ncrs');
    return validateResponse(arrayOf(ncrSchema), res.data).data;
  },
  createNcr: async (data: { description: string; severity?: string }) => {
    const res = await apiClient.post<ApiResponse<Ncr>>('/manufacturing/ncrs', data);
    return validateResponse(ncrSchema, res.data).data;
  },
};
