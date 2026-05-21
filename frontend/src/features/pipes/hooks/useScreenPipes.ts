import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { pipeApi } from '../api/pipeApi';
import type { CreateScreenPipeData, PipeFilterParams } from '../types';

export function useScreenPipes(params?: PipeFilterParams) {
  return useQuery({
    queryKey: ['screen-pipes', params],
    queryFn: () => pipeApi.getScreenPipes(params),
  });
}

export function useScreenPipe(id: number) {
  return useQuery({
    queryKey: ['screen-pipe', id],
    queryFn: () => pipeApi.getScreenPipe(id),
    enabled: !!id,
  });
}

export function useCreateScreenPipe() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateScreenPipeData) => pipeApi.createScreenPipe(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['screen-pipes'] });
    },
  });
}

export function useUpdateScreenPipe(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: Partial<CreateScreenPipeData>) => pipeApi.updateScreenPipe(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['screen-pipes'] });
      qc.invalidateQueries({ queryKey: ['screen-pipe', id] });
    },
  });
}

export function useDeleteScreenPipe() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => pipeApi.deleteScreenPipe(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['screen-pipes'] });
    },
  });
}
