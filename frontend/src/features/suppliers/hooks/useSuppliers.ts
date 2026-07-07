import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { supplierApi } from '../api/supplierApi';
import { supplierQueryKeys } from '../queryKeys';
import type { Supplier, CreateSupplierData, SupplierFilterParams } from '../types';
import type { PaginatedData } from '@/types';

export function useSuppliers(params?: SupplierFilterParams) {
  return useQuery<PaginatedData<Supplier>>({
    queryKey: supplierQueryKeys.list(params),
    queryFn: () => supplierApi.list(params),
  });
}

export function useSupplier(id: number) {
  return useQuery<Supplier>({
    queryKey: supplierQueryKeys.detail(id),
    queryFn: () => supplierApi.getById(id),
    enabled: !!id,
  });
}

export function useCreateSupplier() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateSupplierData) => supplierApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: supplierQueryKeys.all });
    },
  });
}

export function useUpdateSupplier(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: Partial<CreateSupplierData>) => supplierApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: supplierQueryKeys.all });
      qc.invalidateQueries({ queryKey: supplierQueryKeys.detail(id) });
    },
  });
}

export function useDeleteSupplier() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => supplierApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: supplierQueryKeys.all });
    },
  });
}

export function useSupplierSearch(q: string) {
  return useQuery({
    queryKey: supplierQueryKeys.search(q),
    queryFn: () => supplierApi.search(q),
    enabled: q.length > 0,
  });
}

export function useActiveSuppliers() {
  return useQuery({
    queryKey: supplierQueryKeys.active(),
    queryFn: () => supplierApi.listActive(),
  });
}
