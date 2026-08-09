export const hrQueryKeys = {
  employees: {
    all: ['hr-employees'] as const,
    list: (params?: { page?: number; page_size?: number; q?: string }) =>
      [...hrQueryKeys.employees.all, params] as const,
  },
  salaries: {
    all: ['hr-salaries'] as const,
    list: (period?: string) => [...hrQueryKeys.salaries.all, period] as const,
  },
};
