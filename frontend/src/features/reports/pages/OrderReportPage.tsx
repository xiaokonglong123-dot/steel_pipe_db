/**
 * OrderReportPage — Aggregated order statistics by period.
 *
 * Supports filtering by order type (purchase/sales) and period granularity.
 */
import { useState, useEffect } from 'react';
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
import { reportApi } from '../api/reportApi';

const { Title } = Typography;

interface OrderReportRow {
  period: string;
  order_count: number;
  total_amount: number;
  by_status: Record<string, number>;
}

const columns: ColumnsType<OrderReportRow> = [
  { title: '期间', dataIndex: 'period', key: 'period', width: 140 },
  { title: '订单数', dataIndex: 'order_count', key: 'order_count', width: 100 },
  {
    title: '总金额',
    dataIndex: 'total_amount',
    key: 'total_amount',
    width: 140,
    render: (v: number) => `¥${v.toLocaleString()}`,
  },
  {
    title: '按状态',
    dataIndex: 'by_status',
    key: 'by_status',
    render: (by_status: Record<string, number>) => (
      <Space size={[4, 4]} wrap>
        {Object.entries(by_status).map(([status, count]) => (
          <Tag key={status}>
            {status}: {count}
          </Tag>
        ))}
      </Space>
    ),
  },
];

export default function OrderReportPage() {
  const [data, setData] = useState<OrderReportRow[]>([]);
  const [loading, setLoading] = useState(false);
  const [orderType, setOrderType] = useState<string>('purchase');

  useEffect(() => {
    setLoading(true);
    reportApi
      .getOrderReport({ order_type: orderType })
      .then((res) => {
        setData(Array.isArray(res) ? res : res ? [res] : []);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [orderType]);

  const totalOrders = data.reduce((sum, r) => sum + r.order_count, 0);
  const totalAmount = data.reduce((sum, r) => sum + r.total_amount, 0);

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

      <Card>
        <Table<OrderReportRow>
          columns={columns}
          dataSource={data}
          rowKey="period"
          loading={loading}
          pagination={{ pageSize: 20, showTotal: (t) => `共 ${t} 条` }}
          scroll={{ x: 600 }}
        />
      </Card>
    </div>
  );
}
