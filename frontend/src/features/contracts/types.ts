export interface Contract {
  id: number;
  contract_number: string;
  contract_name: string;
  contract_type: 'purchase' | 'sales';
  party_a: string;
  party_b: string;
  sign_date?: string;
  start_date?: string;
  end_date?: string;
  total_amount: number;
  paid_amount: number;
  status: 'draft' | 'active' | 'completed' | 'terminated';
  notes?: string;
  created_at: string;
  updated_at: string;
  items?: ContractItem[];
  payments?: ContractPayment[];
}

export interface ContractItem {
  id: number;
  contract_id: number;
  pipe_type: 'seamless' | 'screen';
  grade: string;
  od: number;
  wt: number;
  length?: number;
  quantity: number;
  unit_price: number;
  total_price: number;
  delivery_date?: string;
  notes?: string;
}

export interface ContractPayment {
  id: number;
  contract_id: number;
  payment_date: string;
  amount: number;
  payment_method?: string;
  reference_number?: string;
  notes?: string;
}

export interface CreateContractData {
  contract_number?: string;
  contract_name: string;
  contract_type: 'purchase' | 'sales';
  party_a: string;
  party_b: string;
  sign_date?: string;
  start_date?: string;
  end_date?: string;
  total_amount: number;
  paid_amount?: number;
  notes?: string;
  items?: Omit<ContractItem, 'id' | 'contract_id'>[];
}

export interface CreateContractItemData {
  pipe_type: 'seamless' | 'screen';
  grade: string;
  od: number;
  wt: number;
  length?: number;
  quantity: number;
  unit_price: number;
  total_price: number;
  delivery_date?: string;
  notes?: string;
}

export interface CreateContractPaymentData {
  payment_date: string;
  amount: number;
  payment_method?: string;
  reference_number?: string;
  notes?: string;
}

export interface ContractFilterParams {
  page?: number;
  page_size?: number;
  q?: string;
  status?: string;
  contract_type?: string;
  sort_by?: string;
  sort_order?: string;
}
