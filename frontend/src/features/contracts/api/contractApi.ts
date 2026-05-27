// Contract API — CRUD + status transitions + contract items + payment milestones
import { z } from 'zod';
import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type {
  Contract,
  ContractItem,
  ContractPayment,
  CreateContractData,
  CreateContractItemData,
  CreateContractPaymentData,
  ContractFilterParams,
} from '../types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { contractSchema, contractItemSchema, contractPaymentSchema } from '@/zod-schemas/orders';

export const contractApi = {
  list: async (params?: ContractFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<Contract>>('/contracts', { params });
    return validateResponse(paginatedDataSchema(contractSchema), res.data.data);
  },

  get: async (id: number) => {
    const res = await apiClient.get<ApiResponse<Contract>>(`/contracts/${id}`);
    return validateResponse(contractSchema, res.data.data);
  },

  create: async (data: CreateContractData) => {
    const res = await apiClient.post<ApiResponse<Contract>>('/contracts', data);
    return validateResponse(contractSchema, res.data.data);
  },

  update: async (id: number, data: Partial<CreateContractData>) => {
    const res = await apiClient.put<ApiResponse<Contract>>(`/contracts/${id}`, data);
    return validateResponse(contractSchema, res.data.data);
  },

  delete: async (id: number) => {
    await apiClient.delete(`/contracts/${id}`);
  },

  // Contract status update (activate/complete/terminate, etc.)
  updateStatus: async (id: number, status: string) => {
    const res = await apiClient.post<ApiResponse<Contract>>(`/contracts/${id}/status`, { status });
    return validateResponse(contractSchema, res.data.data);
  },

  // ━━ Contract items (product details) ━━
  addItem: async (contractId: number, data: CreateContractItemData) => {
    const res = await apiClient.post<ApiResponse<ContractItem>>(`/contracts/${contractId}/items`, data);
    return validateResponse(contractItemSchema, res.data.data);
  },

  updateItem: async (contractId: number, itemId: number, data: Partial<CreateContractItemData>) => {
    const res = await apiClient.put<ApiResponse<ContractItem>>(
      `/contracts/${contractId}/items/${itemId}`,
      data,
    );
    return validateResponse(contractItemSchema, res.data.data);
  },

  deleteItem: async (contractId: number, itemId: number) => {
    await apiClient.delete(`/contracts/${contractId}/items/${itemId}`);
  },

  // ━━ Payment milestones ━━
  listPayments: async (contractId: number) => {
    const res = await apiClient.get<ApiResponse<ContractPayment[]>>(`/contracts/${contractId}/payments`);
    return validateResponse(z.array(contractPaymentSchema), res.data.data);
  },

  addPayment: async (contractId: number, data: CreateContractPaymentData) => {
    const res = await apiClient.post<ApiResponse<ContractPayment>>(`/contracts/${contractId}/payments`, data);
    return validateResponse(contractPaymentSchema, res.data.data);
  },

  updatePayment: async (contractId: number, paymentId: number, data: Partial<CreateContractPaymentData>) => {
    const res = await apiClient.put<ApiResponse<ContractPayment>>(
      `/contracts/${contractId}/payments/${paymentId}`,
      data,
    );
    return validateResponse(contractPaymentSchema, res.data.data);
  },

  deletePayment: async (contractId: number, paymentId: number) => {
    await apiClient.delete(`/contracts/${contractId}/payments/${paymentId}`);
  },
};
