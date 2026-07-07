export interface Contract {
  id: number;
  contract_no: string;
  contract_type: 'purchase' | 'sales';
  title: string;
  party_a: string;
  party_b: string;
  sign_date?: string;
  start_date?: string;
  end_date?: string;
  total_amount?: number;
  status: 'draft' | 'active' | 'completed' | 'terminated' | 'cancelled';
  notes?: string;
  created_by?: number;
  created_at: string;
  updated_at: string;
  deleted_at?: string;
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
  quantity: number;
  unit_price?: number;
  total_price?: number;
  notes?: string;
  created_at: string;
}

export interface ContractPayment {
  id: number;
  contract_id: number;
  due_date: string;
  amount: number;
  payment_type: string;
  is_paid: boolean;
  paid_date?: string;
  notes?: string;
  created_at: string;
}

export interface CreateContractData {
  contract_no?: string;
  contract_type: 'purchase' | 'sales';
  title: string;
  party_a: string;
  party_b: string;
  sign_date?: string;
  start_date?: string;
  end_date?: string;
  total_amount?: number;
  notes?: string;
  items?: Omit<ContractItem, 'id' | 'contract_id' | 'created_at'>[];
}

export interface CreateContractItemData {
  pipe_type: 'seamless' | 'screen';
  grade: string;
  od: number;
  wt: number;
  quantity: number;
  unit_price?: number;
  notes?: string;
}

export interface CreateContractPaymentData {
  due_date: string;
  amount: number;
  payment_type: string;
  notes?: string;
}

/** Wrapped contract detail — matches backend ContractDetailResponse. */
export interface ContractDetail {
  contract: Contract;
  items?: ContractItem[];
  payments?: ContractPayment[];
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
