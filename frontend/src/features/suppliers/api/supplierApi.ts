import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type { Supplier, CreateSupplierData, SupplierFilterParams } from '../types';

export const supplierApi = {
  list: async (params?: SupplierFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<Supplier>>('/suppliers', { params });
    return res.data.data;
  },

  getById: async (id: number) => {
    const res = await apiClient.get<ApiResponse<Supplier>>(`/suppliers/${id}`);
    return res.data.data;
  },

  create: async (data: CreateSupplierData) => {
    const res = await apiClient.post<ApiResponse<Supplier>>('/suppliers', data);
    return res.data.data;
  },

  update: async (id: number, data: Partial<CreateSupplierData>) => {
    const res = await apiClient.put<ApiResponse<Supplier>>(`/suppliers/${id}`, data);
    return res.data.data;
  },

  delete: async (id: number) => {
    await apiClient.delete(`/suppliers/${id}`);
  },

  search: async (q: string) => {
    const res = await apiClient.get<ApiResponse<Supplier[]>>('/suppliers/search', { params: { q } });
    return res.data.data;
  },

  listActive: async () => {
    const res = await apiClient.get<ApiResponse<Supplier[]>>('/suppliers/active');
    return res.data.data;
  },
};
