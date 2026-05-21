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

  const { data: order, isLoading } = useSalesOrder(orderId);
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
    { title: t('Pipe Number'), dataIndex: 'pipe_number', key: 'pipe_number' },
    { title: t('Pipe Type'), dataIndex: 'pipe_type', key: 'pipe_type' },
    { title: t('Grade'), dataIndex: 'grade', key: 'grade' },
    { title: 'OD', dataIndex: 'od', key: 'od', render: (v: number | null) => v ?? '-' },
    { title: 'WT', dataIndex: 'wt', key: 'wt', render: (v: number | null) => v ?? '-' },
    { title: t('Length'), dataIndex: 'length', key: 'length', render: (v: number | null) => v ?? '-' },
    { title: t('Quantity'), dataIndex: 'quantity', key: 'quantity' },
    { title: t('Unit Price'), dataIndex: 'unit_price', key: 'unit_price', render: (v: number) => v.toLocaleString() },
    { title: t('Total Price'), dataIndex: 'total_price', key: 'total_price', render: (v: number) => v.toLocaleString() },
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
        <h2 style={{ margin: 0 }}>{t('Sales Order')} — {order.order_number}</h2>
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
          <Descriptions.Item label={t('Order Number')}>{order.order_number}</Descriptions.Item>
          <Descriptions.Item label={t('Customer')}>{order.customer_name ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('Order Date')}>{order.order_date}</Descriptions.Item>
          <Descriptions.Item label={t('Expected Delivery')}>{order.expected_delivery ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('Total Amount')}>{order.total_amount.toLocaleString()}</Descriptions.Item>
          <Descriptions.Item label={t('Status')}>
            <Tag color={STATUS_COLORS[order.status] ?? 'default'}>{order.status}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label={t('Notes')} span={3}>{order.notes ?? '-'}</Descriptions.Item>
        </Descriptions>
      </Card>

      <Card title={t('Items')} style={{ marginTop: 24 }}>
        <Table
          columns={itemColumns}
          dataSource={order.items}
          rowKey="id"
          pagination={false}
        />
      </Card>

      {nextStatuses.length > 0 && (
        <Card title={t('Status Transition')} style={{ marginTop: 24 }}>
          <Space direction="vertical" style={{ width: '100%' }}>
            <Space>
              <Select
                placeholder={t('Target Status')}
                value={targetStatus}
                onChange={setTargetStatus}
                style={{ width: 200 }}
              >
                {nextStatuses.map((s) => (
                  <Select.Option key={s} value={s}>
                    {s}
                  </Select.Option>
                ))}
              </Select>
              <Input
                placeholder={t('Notes')}
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
                {t('Submit')}
              </Button>
            </Space>
          </Space>
        </Card>
      )}
    </div>
  );
}
