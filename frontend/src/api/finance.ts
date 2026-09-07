import { get, page, post } from "./client"
import type { Page } from "@/types/common"
import type {
  Account,
  AccountPayload,
  Invoice,
  InvoicePayload,
  JournalEntry,
  JournalEntryPayload,
  Payment,
  PaymentPayload,
  TrialBalanceRow,
} from "@/types/finance"

// /accounts 非分页，返回纯数组。
export async function listAccounts(activeOnly = false): Promise<readonly Account[]> {
  return get<readonly Account[]>(`/accounts?active_only=${activeOnly}`)
}
export async function createAccount(payload: AccountPayload): Promise<Account> {
  return post<Account>("/accounts", payload)
}

export async function listJournalEntries(): Promise<Page<JournalEntry>> {
  return page<JournalEntry>("/journal-entries")
}
export async function createJournalEntry(payload: JournalEntryPayload): Promise<JournalEntry> {
  return post<JournalEntry>("/journal-entries", payload)
}
export async function postJournalEntry(id: number): Promise<JournalEntry> {
  return post<JournalEntry>(`/journal-entries/${id}/post`)
}

export async function listInvoices(): Promise<Page<Invoice>> {
  return page<Invoice>("/invoices")
}
export async function createInvoice(payload: InvoicePayload): Promise<Invoice> {
  return post<Invoice>("/invoices", payload)
}

export async function listPayments(): Promise<Page<Payment>> {
  return page<Payment>("/payments")
}
export async function createPayment(payload: PaymentPayload): Promise<Payment> {
  return post<Payment>("/payments", payload)
}

export async function trialBalance(): Promise<readonly TrialBalanceRow[]> {
  return get<readonly TrialBalanceRow[]>("/trial-balance")
}