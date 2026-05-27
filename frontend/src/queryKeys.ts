// Centralized query key factories for TanStack Query
// Keep shapes stable across the app to avoid sprinkling array literals.
export const featureQueryKeys = {
  list: (params: unknown) => ['feature', params] as const,
  listKey: () => ['feature'] as const,
}

export const queryKeys = {
  feature: featureQueryKeys,
}

export default queryKeys
