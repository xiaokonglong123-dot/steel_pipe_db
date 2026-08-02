import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { pipeApi } from '../api/pipeApi';
import { pipeQueryKeys } from '../queryKeys';
import type { CreateWeldedPipeData, PipeFilterParams } from '../types';

export function useWeldedPipes(params?: PipeFilterParams) {
  return useQuery({
    queryKey: pipeQueryKeys.welded.list(params),
    queryFn: () => pipeApi.getWeldedPipes(params),
  });
}

export function useWeldedPipe(id: number) {
  return useQuery({
    queryKey: pipeQueryKeys.welded.detail(id),
    queryFn: () => pipeApi.getWeldedPipe(id),
    enabled: !!id,
  });
}

export function useCreateWeldedPipe() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateWeldedPipeData) => pipeApi.createWeldedPipe(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: pipeQueryKeys.welded.all });
    },
  });
}

export function useUpdateWeldedPipe(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: Partial<CreateWeldedPipeData>) => pipeApi.updateWeldedPipe(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: pipeQueryKeys.welded.all });
      qc.invalidateQueries({ queryKey: pipeQueryKeys.welded.detail(id) });
    },
  });
}

export function useDeleteWeldedPipe() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => pipeApi.deleteWeldedPipe(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: pipeQueryKeys.welded.all });
    },
  });
}
