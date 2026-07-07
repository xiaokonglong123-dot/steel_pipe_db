export interface Customer {
  id: number;
  customer_code: string;
  name: string;
  contact_person?: string;
  phone?: string;
  email?: string;
  address?: string;
  is_active: boolean;
  notes?: string;
  created_at: string;
  updated_at: string;
  deleted_at?: string;
}

export interface CreateCustomerData {
  customer_code?: string;
  name: string;
  contact_person?: string;
  phone?: string;
  email?: string;
  address?: string;
  notes?: string;
}

export type UpdateCustomerData = Partial<CreateCustomerData> & { is_active?: boolean };

export interface CustomerFilterParams extends Record<string, unknown> {
  q?: string;
  is_active?: boolean;
  page?: number;
  page_size?: number;
  sort_by?: string;
  sort_order?: string;
}
