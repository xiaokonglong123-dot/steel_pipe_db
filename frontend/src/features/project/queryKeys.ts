export const projectQueryKeys = {
  all: ['projects'] as const,
  wbs: {
    all: ['wbs'] as const,
    detail: (projectId: number) => [...projectQueryKeys.wbs.all, projectId] as const,
  },
  financials: {
    all: ['fin'] as const,
    detail: (projectId: number) => [...projectQueryKeys.financials.all, projectId] as const,
  },
};
