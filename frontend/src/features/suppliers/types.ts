export interface Supplier {
  id: number;
  code: string;
  name: string;
  contact_person?: string;
  phone?: string;
  email?: string;
  address?: string;
  tax_id?: string;
  bank_info?: string;
  grade_supply?: string;
  status: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateSupplierData {
  code: string;
  name: string;
  contact_person?: string;
  phone?: string;
  email?: string;
  address?: string;
  tax_id?: string;
  bank_info?: string;
  grade_supply?: string;
  status?: string;
  notes?: string;
}

export interface SupplierFilterParams {
  q?: string;
  status?: string;
  page?: number;
  page_size?: number;
  sort_by?: string;
  sort_order?: string;
}
