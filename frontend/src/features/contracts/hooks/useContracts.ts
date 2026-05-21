import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { contractApi } from '../api/contractApi';
import type {
  CreateContractData,
  CreateContractItemData,
  CreateContractPaymentData,
  ContractFilterParams,
} from '../types';

export function useContracts(params?: ContractFilterParams) {
  return useQuery({
    queryKey: ['contracts', params],
    queryFn: () => contractApi.list(params),
  });
}

export function useContract(id: number) {
  return useQuery({
    queryKey: ['contract', id],
    queryFn: () => contractApi.get(id),
    enabled: !!id,
  });
}

export function useCreateContract() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateContractData) => contractApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['contracts'] });
    },
  });
}

export function useUpdateContract(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: Partial<CreateContractData>) => contractApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['contracts'] });
      qc.invalidateQueries({ queryKey: ['contract', id] });
    },
  });
}

export function useDeleteContract() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => contractApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['contracts'] });
    },
  });
}

export function useUpdateContractStatus(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (status: string) => contractApi.updateStatus(id, status),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['contracts'] });
      qc.invalidateQueries({ queryKey: ['contract', id] });
    },
  });
}

export function useCreateContractItem(contractId: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateContractItemData) => contractApi.addItem(contractId, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['contract', contractId] });
    },
  });
}

export function useUpdateContractItem(contractId: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: number; data: Partial<CreateContractItemData> }) =>
      contractApi.updateItem(contractId, id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['contract', contractId] });
    },
  });
}

export function useDeleteContractItem(contractId: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (itemId: number) => contractApi.deleteItem(contractId, itemId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['contract', contractId] });
    },
  });
}

export function useContractPayments(contractId: number) {
  return useQuery({
    queryKey: ['contract-payments', contractId],
    queryFn: () => contractApi.listPayments(contractId),
    enabled: !!contractId,
  });
}

export function useCreateContractPayment(contractId: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateContractPaymentData) => contractApi.addPayment(contractId, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['contract', contractId] });
      qc.invalidateQueries({ queryKey: ['contract-payments', contractId] });
    },
  });
}

export function useUpdateContractPayment(contractId: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: number; data: Partial<CreateContractPaymentData> }) =>
      contractApi.updatePayment(contractId, id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['contract', contractId] });
      qc.invalidateQueries({ queryKey: ['contract-payments', contractId] });
    },
  });
}

export function useDeleteContractPayment(contractId: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (paymentId: number) => contractApi.deletePayment(contractId, paymentId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['contract', contractId] });
      qc.invalidateQueries({ queryKey: ['contract-payments', contractId] });
    },
  });
}
