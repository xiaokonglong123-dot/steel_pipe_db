import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { customerApi } from '../api/customerApi';
import { customerQueryKeys } from '../queryKeys';
import type { CreateCustomerData, CustomerFilterParams } from '../types';

export function useCustomers(params?: CustomerFilterParams) {
  return useQuery({
    queryKey: customerQueryKeys.list(params),
    queryFn: () => customerApi.list(params),
  });
}

export function useCustomer(id: number) {
  return useQuery({
    queryKey: customerQueryKeys.detail(id),
    queryFn: () => customerApi.getById(id),
    enabled: !!id,
  });
}

export function useCreateCustomer() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateCustomerData) => customerApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: customerQueryKeys.all });
    },
  });
}

export function useUpdateCustomer(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: Partial<CreateCustomerData>) => customerApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: customerQueryKeys.all });
      qc.invalidateQueries({ queryKey: customerQueryKeys.detail(id) });
    },
  });
}

export function useDeleteCustomer() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => customerApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: customerQueryKeys.all });
    },
  });
}

export function useCustomerSearch(q: string) {
  return useQuery({
    queryKey: customerQueryKeys.search(q),
    queryFn: () => customerApi.search(q),
    enabled: q.length > 0,
  });
}

export function useActiveCustomers() {
  return useQuery({
    queryKey: customerQueryKeys.active(),
    queryFn: () => customerApi.listActive(),
  });
}
