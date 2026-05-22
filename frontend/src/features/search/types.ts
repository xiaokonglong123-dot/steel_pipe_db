// 搜索结果类型定义
export interface SearchPipeResult {
  id: number;
  pipe_number: string;
  grade: string;
  od: number;
  wt: number;
  length?: number;
  status: string;
  pipe_type: string;
}

export interface SearchInboundResult {
  id: number;
  inbound_no: string;
  inbound_type: string;
  approval_status: string;
  created_at: string;
  notes?: string;
}

export interface SearchOutboundResult {
  id: number;
  outbound_no: string;
  outbound_type: string;
  approval_status: string;
  created_at: string;
  notes?: string;
}

export interface SearchPurchaseOrderResult {
  id: number;
  order_number: string;
  supplier_name?: string;
  status: string;
  order_date: string;
  total_amount?: number;
}

export interface SearchSalesOrderResult {
  id: number;
  order_number: string;
  customer_name?: string;
  status: string;
  order_date: string;
  total_amount?: number;
}

export interface SearchResults {
  pipes: SearchPipeResult[];
  inbound: SearchInboundResult[];
  outbound: SearchOutboundResult[];
  purchases: SearchPurchaseOrderResult[];
  sales: SearchSalesOrderResult[];
}
