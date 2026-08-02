// HR API — employees, attendance, salaries, contracts
import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { z } from 'zod';

export interface HrEmployee {
  id: number;
  employee_no: string;
  name: string;
  gender: string | null;
  phone: string | null;
  department_id: number | null;
  hire_date: string;
  probation_end: string | null;
  status: string;
  base_salary: string | null;
}

export interface HrSalary {
  id: number;
  employee_id: number;
  period: string;
  base_salary: string;
  gross: string;
  net: string;
  status: string;
}

const employeeSchema = z.object({
  id: z.number(),
  employee_no: z.string(),
  name: z.string(),
  gender: z.string().nullable(),
  phone: z.string().nullable(),
  department_id: z.number().nullable(),
  hire_date: z.string(),
  probation_end: z.string().nullable(),
  status: z.string(),
  base_salary: z.string().nullable(),
}).passthrough();

const salarySchema = z.object({
  id: z.number(),
  employee_id: z.number(),
  period: z.string(),
  base_salary: z.string(),
  gross: z.string(),
  net: z.string(),
  status: z.string(),
}).passthrough();

const arrayOf = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: z.array(item) }).passthrough();

export const hrApi = {
  listEmployees: async (params?: { page?: number; page_size?: number; q?: string }) => {
    const res = await apiClient.get<PaginatedResponse<HrEmployee>>('/hr/employees', params);
    return validateResponse(paginatedDataSchema(employeeSchema), res.data);
  },

  createEmployee: async (data: {
    employee_no: string;
    name: string;
    phone?: string;
    hire_date: string;
    base_salary?: number;
  }) => {
    const res = await apiClient.post<ApiResponse<HrEmployee>>('/hr/employees', data);
    return validateResponse(employeeSchema, res.data).data;
  },

  terminateEmployee: async (id: number, reason?: string) => {
    const res = await apiClient.post<ApiResponse<HrEmployee>>(`/hr/employees/${id}/terminate`, { reason });
    return validateResponse(employeeSchema, res.data).data;
  },

  listSalaries: async (period?: string) => {
    const res = await apiClient.get<ApiResponse<HrSalary[]>>('/hr/salaries', { period });
    return validateResponse(arrayOf(salarySchema), res.data).data;
  },

  generateSalaries: async (period: string) => {
    const res = await apiClient.post<ApiResponse<HrSalary[]>>('/hr/salaries', { period });
    return validateResponse(arrayOf(salarySchema), res.data).data;
  },
};
