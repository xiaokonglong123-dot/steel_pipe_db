import client from '../../../api/client';
import { ApiResponse, ApiListResponse } from '../../../shared/types';

export interface Customer {
  id: string;
  name: string;
  contact_person?: string;
  phone?: string;
  email?: string;
  address?: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CustomerFilter {
  page?: number;
  page_size?: number;
  search?: string;
  is_active?: boolean;
}

export interface CustomerPayload {
  name: string;
  contact_person?: string;
  phone?: string;
  email?: string;
  address?: string;
  is_active?: boolean;
}

const BASE = '/customers';

export const customerApi = {
  list: (params?: CustomerFilter) =>
    client.get<ApiListResponse<Customer>>(BASE, { params }),

  create: (data: CustomerPayload) =>
    client.post<ApiResponse<Customer>>(BASE, data),

  get: (id: string) =>
    client.get<ApiResponse<Customer>>(`${BASE}/${id}`),

  update: (id: string, data: CustomerPayload) =>
    client.put<ApiResponse<Customer>>(`${BASE}/${id}`, data),

  delete: (id: string) =>
    client.delete(`${BASE}/${id}`),
};
