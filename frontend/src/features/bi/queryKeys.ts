export const biQueryKeys = {
  financeSummary: ['bi-finance-summary'] as const,
  inventoryValue: ['bi-inventory-value'] as const,
  salesTrend: (months = 12) => ['bi-sales-trend', months] as const,
};
