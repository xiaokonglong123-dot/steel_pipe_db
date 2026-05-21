import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type { Customer, CreateCustomerData, CustomerFilterParams } from '../types';

export const customerApi = {
  list: async (params?: CustomerFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<Customer>>('/customers', { params });
    return res.data.data;
  },

  getById: async (id: number) => {
    const res = await apiClient.get<ApiResponse<Customer>>(`/customers/${id}`);
    return res.data.data;
  },

  create: async (data: CreateCustomerData) => {
    const res = await apiClient.post<ApiResponse<Customer>>('/customers', data);
    return res.data.data;
  },

  update: async (id: number, data: Partial<CreateCustomerData>) => {
    const res = await apiClient.put<ApiResponse<Customer>>(`/customers/${id}`, data);
    return res.data.data;
  },

  delete: async (id: number) => {
    await apiClient.delete(`/customers/${id}`);
  },

  search: async (q: string) => {
    const res = await apiClient.get<ApiResponse<Customer[]>>('/customers/search', { params: { q } });
    return res.data.data;
  },

  listActive: async () => {
    const res = await apiClient.get<ApiResponse<Customer[]>>('/customers/active');
    return res.data.data;
  },
};
