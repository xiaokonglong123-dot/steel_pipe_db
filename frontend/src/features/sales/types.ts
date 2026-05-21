export interface SalesOrder {
  id: number;
  order_number: string;
  customer_id: number;
  customer_name?: string;
  order_date: string;
  expected_delivery?: string;
  status: string;
  total_amount: number;
  notes?: string;
  items: SalesOrderItem[];
  created_at: string;
  updated_at: string;
}

export interface SalesOrderItem {
  id: number;
  sales_order_id: number;
  pipe_id: number;
  pipe_number?: string;
  pipe_type?: string;
  grade?: string;
  od?: number;
  wt?: number;
  length?: number;
  quantity: number;
  unit_price: number;
  total_price: number;
  notes?: string;
}

export interface CreateSalesOrderData {
  customer_id: number;
  customer_name?: string;
  order_date?: string;
  expected_delivery?: string;
  notes?: string;
  items: CreateSalesOrderItemData[];
}

export interface CreateSalesOrderItemData {
  pipe_id: number;
  pipe_number?: string;
  pipe_type?: string;
  grade?: string;
  od?: number;
  wt?: number;
  length?: number;
  quantity: number;
  unit_price: number;
  total_price?: number;
  notes?: string;
}

export interface SalesOrderFilterParams {
  page?: number;
  page_size?: number;
  status?: string;
  customer_id?: number;
  q?: string;
  sort_by?: string;
  sort_order?: string;
}

export interface SalesOrderStatusTransitionRequest {
  status: string;
  notes?: string;
}

export interface UpdateSalesOrderItemData {
  quantity?: number;
  unit_price?: number;
  notes?: string;
}
