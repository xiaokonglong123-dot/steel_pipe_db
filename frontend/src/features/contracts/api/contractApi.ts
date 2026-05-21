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
    return validateResponse(res.data.data, paginatedDataSchema(contractSchema));
  },

  get: async (id: number) => {
    const res = await apiClient.get<ApiResponse<Contract>>(`/contracts/${id}`);
    return validateResponse(res.data.data, contractSchema);
  },

  create: async (data: CreateContractData) => {
    const res = await apiClient.post<ApiResponse<Contract>>('/contracts', data);
    return validateResponse(res.data.data, contractSchema);
  },

  update: async (id: number, data: Partial<CreateContractData>) => {
    const res = await apiClient.put<ApiResponse<Contract>>(`/contracts/${id}`, data);
    return validateResponse(res.data.data, contractSchema);
  },

  delete: async (id: number) => {
    await apiClient.delete(`/contracts/${id}`);
  },

  updateStatus: async (id: number, status: string) => {
    const res = await apiClient.post<ApiResponse<Contract>>(`/contracts/${id}/status`, { status });
    return validateResponse(res.data.data, contractSchema);
  },

  addItem: async (contractId: number, data: CreateContractItemData) => {
    const res = await apiClient.post<ApiResponse<ContractItem>>(`/contracts/${contractId}/items`, data);
    return validateResponse(res.data.data, contractItemSchema);
  },

  updateItem: async (contractId: number, itemId: number, data: Partial<CreateContractItemData>) => {
    const res = await apiClient.put<ApiResponse<ContractItem>>(
      `/contracts/${contractId}/items/${itemId}`,
      data,
    );
    return validateResponse(res.data.data, contractItemSchema);
  },

  deleteItem: async (contractId: number, itemId: number) => {
    await apiClient.delete(`/contracts/${contractId}/items/${itemId}`);
  },

  listPayments: async (contractId: number) => {
    const res = await apiClient.get<ApiResponse<ContractPayment[]>>(`/contracts/${contractId}/payments`);
    return validateResponse(res.data.data, z.array(contractPaymentSchema));
  },

  addPayment: async (contractId: number, data: CreateContractPaymentData) => {
    const res = await apiClient.post<ApiResponse<ContractPayment>>(`/contracts/${contractId}/payments`, data);
    return validateResponse(res.data.data, contractPaymentSchema);
  },

  updatePayment: async (contractId: number, paymentId: number, data: Partial<CreateContractPaymentData>) => {
    const res = await apiClient.put<ApiResponse<ContractPayment>>(
      `/contracts/${contractId}/payments/${paymentId}`,
      data,
    );
    return validateResponse(res.data.data, contractPaymentSchema);
  },

  deletePayment: async (contractId: number, paymentId: number) => {
    await apiClient.delete(`/contracts/${contractId}/payments/${paymentId}`);
  },
};
