// 销售订单详情页 — 完整订单信息 + 行项表格 + 状态流转操作
import { Button, Descriptions, Space, Tag, Card, Table, Select, Input, message } from 'antd';
import { EditOutlined, ArrowLeftOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useState } from 'react';
import { useSalesOrder, useTransitionSalesOrder } from '../hooks/useSales';

const STATUS_COLORS: Record<string, string> = {
  draft: 'default',
  pending: 'blue',
  approved: 'cyan',
  delivered: 'green',
  invoiced: 'purple',
  cancelled: 'red',
};

const NEXT_STATUSES: Record<string, string[]> = {
  draft: ['pending', 'cancelled'],
  pending: ['approved', 'cancelled'],
  approved: ['delivered', 'cancelled'],
  delivered: ['invoiced'],
  invoiced: [],
  cancelled: [],
};

export default function SalesOrderDetailPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const orderId = Number(id);
  const [targetStatus, setTargetStatus] = useState<string | undefined>();
  const [transitionNotes, setTransitionNotes] = useState('');

  const { data: detail, isLoading } = useSalesOrder(orderId);
  const order = detail?.order;
  const items = detail?.items ?? [];
  const transitionMutation = useTransitionSalesOrder(orderId);

  const handleTransition = async () => {
    if (!targetStatus) return;
    try {
      await transitionMutation.mutateAsync({ status: targetStatus, notes: transitionNotes || undefined });
      message.success(t('common.operate_success'));
      setTargetStatus(undefined);
      setTransitionNotes('');
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  const itemColumns = [
    { title: t('pipes.pipe_number'), dataIndex: 'pipe_number', key: 'pipe_number' },
    { title: t('pipes.pipe_type'), dataIndex: 'pipe_type', key: 'pipe_type' },
    { title: t('pipes.grade'), dataIndex: 'grade', key: 'grade' },
  { title: t('sales.od'), dataIndex: 'od', key: 'od', render: (v: number | null) => v ?? '-' },
  { title: t('sales.wt'), dataIndex: 'wt', key: 'wt', render: (v: number | null) => v ?? '-' },
  { title: t('sales.length'), dataIndex: 'length', key: 'length', render: (v: number | null) => v ?? '-' },
  { title: t('sales.quantity'), dataIndex: 'quantity', key: 'quantity' },
    { title: t('sales.unit_price'), dataIndex: 'unit_price', key: 'unit_price', render: (v: number) => v.toLocaleString() },
    { title: t('sales.total_price'), dataIndex: 'total_price', key: 'total_price', render: (v: number) => v.toLocaleString() },
  ];

  if (isLoading) {
    return <div>{t('common.loading')}</div>;
  }

  if (!order) {
    return <div>{t('common.no_data')}</div>;
  }

  const nextStatuses = NEXT_STATUSES[order.status] ?? [];

  return (
    <div>
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: 24,
        }}
      >
        <h2 style={{ margin: 0 }}>{t('sales.sales_order')} — {order.order_number}</h2>
        <Space>
          <Button
            type="primary"
            icon={<EditOutlined />}
            onClick={() => navigate(`/sales/${order.id}/edit`)}
          >
            {t('common.edit')}
          </Button>
          <Button
            icon={<ArrowLeftOutlined />}
            onClick={() => navigate('/sales')}
          >
            {t('common.back')}
          </Button>
        </Space>
      </div>

      <Card>
        <Descriptions bordered column={{ xs: 1, sm: 2, lg: 3 }}>
          <Descriptions.Item label={t('sales.order_number')}>{order.order_number}</Descriptions.Item>
          <Descriptions.Item label={t('sales.customer')}>{order.customer_name ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('sales.order_date')}>{order.order_date}</Descriptions.Item>
          <Descriptions.Item label={t('sales.expected_delivery')}>{order.expected_delivery ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('sales.total_amount')}>{order.total_amount.toLocaleString()}</Descriptions.Item>
          <Descriptions.Item label={t('sales.status')}>
            <Tag color={STATUS_COLORS[order.status] ?? 'default'}>{t('sales.status.' + order.status)}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label={t('common.notes')} span={3}>{order.notes ?? '-'}</Descriptions.Item>
        </Descriptions>
      </Card>

      <Card title={t('sales.items')} style={{ marginTop: 24 }}>
        <Table
          columns={itemColumns}
          dataSource={items}
          rowKey="id"
          pagination={false}
        />
      </Card>

      {nextStatuses.length > 0 && (
        <Card title={t('sales.status_transition')} style={{ marginTop: 24 }}>
          <Space direction="vertical" style={{ width: '100%' }}>
            <Space>
              <Select
                placeholder={t('sales.target_status')}
                value={targetStatus}
                onChange={setTargetStatus}
                style={{ width: 200 }}
              >
                {nextStatuses.map((s) => (
                  <Select.Option key={s} value={s}>
                    {t('sales.status.' + s)}
                  </Select.Option>
                ))}
              </Select>
              <Input
                placeholder={t('common.notes')}
                value={transitionNotes}
                onChange={(e) => setTransitionNotes(e.target.value)}
                style={{ width: 300 }}
              />
              <Button
                type="primary"
                onClick={handleTransition}
                loading={transitionMutation.isPending}
                disabled={!targetStatus}
              >
                {t('sales.submit')}
              </Button>
            </Space>
          </Space>
        </Card>
      )}
    </div>
  );
}
