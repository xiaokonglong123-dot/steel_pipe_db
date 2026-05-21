import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { pipeApi } from '../api/pipeApi';
import type { CreateSeamlessPipeData, PipeFilterParams } from '../types';

export function useSeamlessPipes(params?: PipeFilterParams) {
  return useQuery({
    queryKey: ['seamless-pipes', params],
    queryFn: () => pipeApi.getSeamlessPipes(params),
  });
}

export function useSeamlessPipe(id: number) {
  return useQuery({
    queryKey: ['seamless-pipe', id],
    queryFn: () => pipeApi.getSeamlessPipe(id),
    enabled: !!id,
  });
}

export function useCreateSeamlessPipe() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateSeamlessPipeData) => pipeApi.createSeamlessPipe(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['seamless-pipes'] });
    },
  });
}

export function useUpdateSeamlessPipe(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: Partial<CreateSeamlessPipeData>) => pipeApi.updateSeamlessPipe(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['seamless-pipes'] });
      qc.invalidateQueries({ queryKey: ['seamless-pipe', id] });
    },
  });
}

export function useDeleteSeamlessPipe() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => pipeApi.deleteSeamlessPipe(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['seamless-pipes'] });
    },
  });
}
