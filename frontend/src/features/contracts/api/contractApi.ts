import client from '../../../api/client';
import { ApiResponse, ApiListResponse } from '../../../shared/types';

export interface Contract {
  id: string;
  contract_no: string;
  contract_type: string;
  party_id: string;
  party_name: string;
  total_amount: number;
  status: string;
  sign_date?: string;
  effective_date?: string;
  expiry_date?: string;
  notes?: string;
  operator_id?: string;
  created_at: string;
}

export interface ContractItem {
  id: string;
  contract_id: string;
  description: string;
  spec?: string;
  quantity: number;
  unit_price: number;
  amount: number;
  delivery_date?: string;
}

export interface ContractPayment {
  id: string;
  contract_id: string;
  stage: string;
  amount: number;
  due_date?: string;
  paid_date?: string;
  status: string;
}

export interface ContractFilter {
  page?: number;
  page_size?: number;
  contract_type?: string;
  status?: string;
  date_from?: string;
  date_to?: string;
}

export const contractApi = {
  listContracts: (params?: ContractFilter) =>
    client.get<ApiListResponse<Contract>>('/contracts', { params }),

  getContract: (id: string) =>
    client.get<ApiResponse<Contract>>(`/contracts/${id}`),

  createContract: (data: Partial<Contract>) =>
    client.post<ApiResponse<Contract>>('/contracts', data),

  updateContract: (id: string, data: Partial<Contract>) =>
    client.put<ApiResponse<Contract>>(`/contracts/${id}`, data),

  deleteContract: (id: string) =>
    client.delete(`/contracts/${id}`),
};
