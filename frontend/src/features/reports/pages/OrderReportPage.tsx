/**
 * OrderReportPage — Aggregated order statistics by period.
 *
 * Supports filtering by order type (purchase/sales) and period granularity.
 * Backend returns: { type, period, orders, status_distribution, top_customers|top_suppliers }
 * See backend/src/services/report_service.rs → order_report()
 */
import { useState } from 'react';
import {
  Card,
  Table,
  Typography,
  Select,
  Space,
  Tag,
  Statistic,
  Row,
  Col,
} from 'antd';
import { ShoppingCartOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { useOrderReport } from '../hooks/useReports';
import type { OrderReportOrder, StatusDistribution } from '../types';

const { Title } = Typography;

const orderColumns: ColumnsType<OrderReportOrder> = [
  { title: '期间', dataIndex: 'period', key: 'period', width: 140 },
  { title: '订单数', dataIndex: 'order_count', key: 'order_count', width: 100 },
  {
    title: '总金额',
    dataIndex: 'total_amount',
    key: 'total_amount',
    width: 140,
    render: (v: number) => `¥${v.toLocaleString()}`,
  },
];

export default function OrderReportPage() {
  const [orderType, setOrderType] = useState<string>('purchase');

  // [#6] send `type` (not `order_type`) and `period` to match backend expectations
  const { data, isLoading } = useOrderReport({ type: orderType, period: 'monthly' });

  const totalOrders = data?.orders?.reduce((sum, r) => sum + r.order_count, 0) ?? 0;
  const totalAmount = data?.orders?.reduce((sum, r) => sum + r.total_amount, 0) ?? 0;

  return (
    <div style={{ padding: 24 }}>
      <Title level={3}>
        <ShoppingCartOutlined style={{ marginRight: 8 }} />
        订单报表
      </Title>

      <Card style={{ marginBottom: 16 }}>
        <Space>
          <span>订单类型：</span>
          <Select
            value={orderType}
            onChange={setOrderType}
            style={{ width: 160 }}
            options={[
              { value: 'purchase', label: '采购订单' },
              { value: 'sales', label: '销售订单' },
            ]}
          />
          <span style={{ marginLeft: 16, color: '#888' }}>
            类型: {data?.type ?? '-'} / 周期: {data?.period ?? '-'}
          </span>
        </Space>
      </Card>

      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={12}>
          <Card>
            <Statistic title="总订单数" value={totalOrders} />
          </Card>
        </Col>
        <Col span={12}>
          <Card>
            <Statistic title="总金额" value={totalAmount} prefix="¥" />
          </Card>
        </Col>
      </Row>

      {/* Status Distribution */}
      {data?.status_distribution && data.status_distribution.length > 0 && (
        <Card title="状态分布" style={{ marginBottom: 16 }}>
          <Space size={[8, 8]} wrap>
            {data.status_distribution.map((s: StatusDistribution) => (
              <Tag key={s.status} color="blue">
                {s.status}: {s.count}
              </Tag>
            ))}
          </Space>
        </Card>
      )}

      {/* Top Customers (sales) */}
      {orderType === 'sales' && data?.top_customers && data.top_customers.length > 0 && (
        <Card title="Top 客户" style={{ marginBottom: 16 }}>
          <Table
            columns={[
              { title: '客户', dataIndex: 'customer', key: 'customer', width: 200 },
              { title: '订单数', dataIndex: 'order_count', key: 'order_count', width: 100 },
              { title: '总金额', dataIndex: 'total_amount', key: 'total_amount', render: (v: number) => `¥${v.toLocaleString()}` },
            ]}
            dataSource={data.top_customers}
            rowKey="customer"
            pagination={false}
            size="small"
          />
        </Card>
      )}

      {/* Top Suppliers (purchase) */}
      {orderType === 'purchase' && data?.top_suppliers && data.top_suppliers.length > 0 && (
        <Card title="Top 供应商" style={{ marginBottom: 16 }}>
          <Table
            columns={[
              { title: '供应商', dataIndex: 'supplier', key: 'supplier', width: 200 },
              { title: '订单数', dataIndex: 'order_count', key: 'order_count', width: 100 },
              { title: '总金额', dataIndex: 'total_amount', key: 'total_amount', render: (v: number) => `¥${v.toLocaleString()}` },
            ]}
            dataSource={data.top_suppliers}
            rowKey="supplier"
            pagination={false}
            size="small"
          />
        </Card>
      )}

      {/* Orders Table */}
      <Card>
        <Table<OrderReportOrder>
          columns={orderColumns}
          dataSource={data?.orders}
          rowKey="period"
          loading={isLoading}
          pagination={{ pageSize: 20, showTotal: (t) => `共 ${t} 条` }}
          scroll={{ x: 400 }}
        />
      </Card>
    </div>
  );
}
