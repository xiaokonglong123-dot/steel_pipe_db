import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type { Customer, CreateCustomerData, CustomerFilterParams } from '../types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { z } from 'zod';
import { customerSchema } from '@/zod-schemas/core';

export const customerApi = {
  list: async (params?: CustomerFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<Customer>>('/customers', { params });
    return validateResponse(res.data.data, paginatedDataSchema(customerSchema));
  },

  getById: async (id: number) => {
    const res = await apiClient.get<ApiResponse<Customer>>(`/customers/${id}`);
    return validateResponse(res.data.data, customerSchema);
  },

  create: async (data: CreateCustomerData) => {
    const res = await apiClient.post<ApiResponse<Customer>>('/customers', data);
    return validateResponse(res.data.data, customerSchema);
  },

  update: async (id: number, data: Partial<CreateCustomerData>) => {
    const res = await apiClient.put<ApiResponse<Customer>>(`/customers/${id}`, data);
    return validateResponse(res.data.data, customerSchema);
  },

  delete: async (id: number) => {
    await apiClient.delete(`/customers/${id}`);
  },

  search: async (q: string) => {
    const res = await apiClient.get<ApiResponse<Customer[]>>('/customers/search', { params: { q } });
    return validateResponse(res.data.data, z.array(customerSchema));
  },

  listActive: async () => {
    const res = await apiClient.get<ApiResponse<Customer[]>>('/customers/active');
    return validateResponse(res.data.data, z.array(customerSchema));
  },
};
