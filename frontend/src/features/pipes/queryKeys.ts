import type { PipeFilterParams } from './types';

export const pipeQueryKeys = {
  seamless: {
    all: ['seamless-pipes'] as const,
    list: (params?: PipeFilterParams) => [...pipeQueryKeys.seamless.all, params] as const,
    detail: (id: number) => ['seamless-pipe', id] as const,
  },
  screen: {
    all: ['screen-pipes'] as const,
    list: (params?: PipeFilterParams) => [...pipeQueryKeys.screen.all, params] as const,
    detail: (id: number) => ['screen-pipe', id] as const,
  },
};
