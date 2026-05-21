import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type { Supplier, CreateSupplierData, SupplierFilterParams } from '../types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { z } from 'zod';
import { supplierSchema } from '@/zod-schemas/core';

export const supplierApi = {
  list: async (params?: SupplierFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<Supplier>>('/suppliers', { params });
    return validateResponse(res.data.data, paginatedDataSchema(supplierSchema));
  },

  getById: async (id: number) => {
    const res = await apiClient.get<ApiResponse<Supplier>>(`/suppliers/${id}`);
    return validateResponse(res.data.data, supplierSchema);
  },

  create: async (data: CreateSupplierData) => {
    const res = await apiClient.post<ApiResponse<Supplier>>('/suppliers', data);
    return validateResponse(res.data.data, supplierSchema);
  },

  update: async (id: number, data: Partial<CreateSupplierData>) => {
    const res = await apiClient.put<ApiResponse<Supplier>>(`/suppliers/${id}`, data);
    return validateResponse(res.data.data, supplierSchema);
  },

  delete: async (id: number) => {
    await apiClient.delete(`/suppliers/${id}`);
  },

  search: async (q: string) => {
    const res = await apiClient.get<ApiResponse<Supplier[]>>('/suppliers/search', { params: { q } });
    return validateResponse(res.data.data, z.array(supplierSchema));
  },

  listActive: async () => {
    const res = await apiClient.get<ApiResponse<Supplier[]>>('/suppliers/active');
    return validateResponse(res.data.data, z.array(supplierSchema));
  },
};
