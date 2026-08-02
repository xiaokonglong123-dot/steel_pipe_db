// Finance API — accounts, journal entries, invoices, payments
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';

export interface Account {
  id: number;
  code: string;
  name: string;
  account_type: string;
  is_active: boolean;
}

export interface JournalEntry {
  id: number;
  entry_no: string;
  entry_date: string;
  description: string | null;
  status: string;
}

export interface FinanceInvoice {
  id: number;
  invoice_no: string;
  invoice_type: string;
  party_id: number;
  total_amount: string;
  status: string;
  due_date: string | null;
}

export interface FinancePayment {
  id: number;
  payment_no: string;
  direction: string;
  amount: string;
  method: string;
  invoice_id: number | null;
}

const accountSchema = z.object({
  id: z.number(),
  code: z.string(),
  name: z.string(),
  account_type: z.string(),
  is_active: z.boolean(),
}).passthrough();

const entrySchema = z.object({
  id: z.number(),
  entry_no: z.string(),
  entry_date: z.string(),
  description: z.string().nullable(),
  status: z.string(),
}).passthrough();

const invoiceSchema = z.object({
  id: z.number(),
  invoice_no: z.string(),
  invoice_type: z.string(),
  party_id: z.number(),
  total_amount: z.string(),
  status: z.string(),
  due_date: z.string().nullable(),
}).passthrough();

const paymentSchema = z.object({
  id: z.number(),
  payment_no: z.string(),
  direction: z.string(),
  amount: z.string(),
  method: z.string(),
  invoice_id: z.number().nullable(),
}).passthrough();

const arrayOf = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: z.array(item) }).passthrough();

export const financeApi = {
  listAccounts: async () => {
    const res = await apiClient.get<ApiResponse<Account[]>>('/chart-of-accounts');
    return validateResponse(arrayOf(accountSchema), res.data).data;
  },
  createAccount: async (data: { code: string; name: string; account_type: string }) => {
    const res = await apiClient.post<ApiResponse<Account>>('/chart-of-accounts', data);
    return validateResponse(accountSchema, res.data).data;
  },
  listEntries: async () => {
    const res = await apiClient.get<ApiResponse<JournalEntry[]>>('/journal-entries');
    return validateResponse(arrayOf(entrySchema), res.data).data;
  },
  createEntry: async (data: {
    entry_date: string;
    description?: string;
    details: { account_id: number; debit?: number; credit?: number }[];
  }) => {
    const res = await apiClient.post<ApiResponse<JournalEntry>>('/journal-entries', data);
    return validateResponse(entrySchema, res.data).data;
  },
  listInvoices: async (invoice_type?: string) => {
    const res = await apiClient.get<ApiResponse<FinanceInvoice[]>>('/invoices', { invoice_type });
    return validateResponse(arrayOf(invoiceSchema), res.data).data;
  },
  createInvoice: async (data: { invoice_type: string; party_id: number; amount: number; tax_amount?: number }) => {
    const res = await apiClient.post<ApiResponse<FinanceInvoice>>('/invoices', data);
    return validateResponse(invoiceSchema, res.data).data;
  },
  confirmInvoice: async (id: number) => {
    const res = await apiClient.post<ApiResponse<FinanceInvoice>>(`/invoices/${id}/confirm`);
    return validateResponse(invoiceSchema, res.data).data;
  },
  listPayments: async () => {
    const res = await apiClient.get<ApiResponse<FinancePayment[]>>('/payments');
    return validateResponse(arrayOf(paymentSchema), res.data).data;
  },
  createPayment: async (data: { invoice_id?: number; direction: string; amount: number; method?: string }) => {
    const res = await apiClient.post<ApiResponse<FinancePayment>>('/payments', data);
    return validateResponse(paymentSchema, res.data).data;
  },
  trialBalance: async () => {
    const res = await apiClient.get<ApiResponse<{ code: string; name: string; debit: string; credit: string }[]>>('/finance/trial-balance');
    return validateResponse(
      arrayOf(z.object({ code: z.string(), name: z.string(), debit: z.string(), credit: z.string() }).passthrough()),
      res.data,
    ).data;
  },
};
