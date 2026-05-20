import client from '../../../api/client';
import { ApiResponse, ApiListResponse } from '../../../shared/types';

export interface Supplier {
  id: string;
  name: string;
  contact_person?: string;
  phone?: string;
  email?: string;
  address?: string;
  cert_info?: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface SupplierFilter {
  page?: number;
  page_size?: number;
  search?: string;
  is_active?: boolean;
}

export interface SupplierPayload {
  name: string;
  contact_person?: string;
  phone?: string;
  email?: string;
  address?: string;
  cert_info?: string;
  is_active?: boolean;
}

const BASE = '/suppliers';

export const supplierApi = {
  list: (params?: SupplierFilter) =>
    client.get<ApiListResponse<Supplier>>(BASE, { params }),

  create: (data: SupplierPayload) =>
    client.post<ApiResponse<Supplier>>(BASE, data),

  get: (id: string) =>
    client.get<ApiResponse<Supplier>>(`${BASE}/${id}`),

  update: (id: string, data: SupplierPayload) =>
    client.put<ApiResponse<Supplier>>(`${BASE}/${id}`, data),

  delete: (id: string) =>
    client.delete(`${BASE}/${id}`),
};
