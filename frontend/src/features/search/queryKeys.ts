export const searchQueryKeys = {
  pipes: (query: string) => ['search', 'pipes', query] as const,
  inbound: (query: string) => ['search', 'inbound', query] as const,
  outbound: (query: string) => ['search', 'outbound', query] as const,
  purchases: (query: string) => ['search', 'purchases', query] as const,
  sales: (query: string) => ['search', 'sales', query] as const,
};
