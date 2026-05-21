// 采购订单详情页 — 完整订单信息 + 行项表格 + 审核状态流转操作
import { useState } from 'react';
import { Button, Descriptions, Space, Tag, Card, Table, Select, message, Modal, Input } from 'antd';
import { EditOutlined, ArrowLeftOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { usePurchase, useTransitionPurchaseOrder } from '../hooks/usePurchases';

const STATUS_COLORS: Record<string, string> = {
  draft: 'default',
  pending: 'orange',
  approved: 'blue',
  received: 'green',
  cancelled: 'red',
};

const STATUS_TRANSITIONS: Record<string, string[]> = {
  draft: ['pending', 'cancelled'],
  pending: ['approved', 'cancelled'],
  approved: ['received'],
};

export default function PurchaseOrderDetailPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const orderId = Number(id);

  const { data: order, isLoading } = usePurchase(orderId);
  const transitionMutation = useTransitionPurchaseOrder(orderId);

  const [transitionModalOpen, setTransitionModalOpen] = useState(false);
  const [targetStatus, setTargetStatus] = useState<string>('');
  const [transitionNotes, setTransitionNotes] = useState('');

  const handleTransition = async () => {
    try {
      await transitionMutation.mutateAsync({
        status: targetStatus,
        notes: transitionNotes || undefined,
      });
      message.success(t('common.operate_success'));
      setTransitionModalOpen(false);
      setTargetStatus('');
      setTransitionNotes('');
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  if (isLoading) {
    return <div>{t('common.loading')}</div>;
  }

  if (!order) {
    return <div>{t('common.no_data')}</div>;
  }

  const availableTransitions = STATUS_TRANSITIONS[order.status] ?? [];
  const showTransitionBtn = availableTransitions.length > 0;

  const itemColumns = [
    {
      title: t('purchases.pipe_type'),
      dataIndex: 'pipe_type',
      key: 'pipe_type',
    },
    {
      title: t('purchases.grade'),
      dataIndex: 'grade',
      key: 'grade',
      render: (grade: string) => <Tag color="blue">{grade}</Tag>,
    },
    {
      title: t('purchases.od'),
      dataIndex: 'od',
      key: 'od',
    },
    {
      title: t('purchases.wt'),
      dataIndex: 'wt',
      key: 'wt',
    },
    {
      title: t('purchases.length'),
      dataIndex: 'length',
      key: 'length',
      render: (val: number | undefined) => val ?? '-',
    },
    {
      title: t('purchases.quantity'),
      dataIndex: 'quantity',
      key: 'quantity',
    },
    {
      title: t('purchases.unit_price'),
      dataIndex: 'unit_price',
      key: 'unit_price',
      render: (val: number) => `¥${val.toFixed(2)}`,
    },
    {
      title: t('purchases.total_price'),
      dataIndex: 'total_price',
      key: 'total_price',
      render: (val: number) => `¥${val.toFixed(2)}`,
    },
    {
      title: t('purchases.notes'),
      dataIndex: 'notes',
      key: 'notes',
      render: (val: string | undefined) => val ?? '-',
    },
  ];

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
        <h2 style={{ margin: 0 }}>{t('purchases.purchase_order')} — {order.order_number}</h2>
        <Space>
          {showTransitionBtn && (
            <Button
              type="primary"
              onClick={() => setTransitionModalOpen(true)}
            >
              {t('purchases.update_status')}
            </Button>
          )}
          <Button
            icon={<EditOutlined />}
            onClick={() => navigate(`/purchases/${order.id}/edit`)}
          >
            {t('common.edit')}
          </Button>
          <Button
            icon={<ArrowLeftOutlined />}
            onClick={() => navigate('/purchases')}
          >
            {t('common.back')}
          </Button>
        </Space>
      </div>

      <Card title={t('purchases.order_info')} style={{ marginBottom: 24 }}>
        <Descriptions bordered column={{ xs: 1, sm: 2, lg: 3 }}>
          <Descriptions.Item label={t('purchases.order_number')}>{order.order_number}</Descriptions.Item>
          <Descriptions.Item label={t('purchases.supplier')}>{order.supplier_name}</Descriptions.Item>
          <Descriptions.Item label={t('purchases.order_date')}>{order.order_date}</Descriptions.Item>
          <Descriptions.Item label={t('purchases.expected_delivery')}>{order.expected_date ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('purchases.status')}>
            <Tag color={STATUS_COLORS[order.status] ?? 'default'}>{t(`purchases.status.${order.status}`)}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label={t('purchases.total_amount')}>¥{order.total_amount.toFixed(2)}</Descriptions.Item>
          <Descriptions.Item label={t('purchases.notes')} span={3}>
            {order.notes ?? '-'}
          </Descriptions.Item>
        </Descriptions>
      </Card>

      <Card title={t('purchases.items')}>
        <Table
          columns={itemColumns}
          dataSource={order.items}
          rowKey="id"
          pagination={false}
          summary={() => {
            const total = order.items.reduce(
              (sum, item) => sum + item.total_price,
              0,
            );
            return (
              <Table.Summary.Row>
                <Table.Summary.Cell index={0} colSpan={7}>
                  <strong>{t('purchases.total')}</strong>
                </Table.Summary.Cell>
                <Table.Summary.Cell index={1}>
                  <strong>¥{total.toFixed(2)}</strong>
                </Table.Summary.Cell>
                <Table.Summary.Cell index={2} />
              </Table.Summary.Row>
            );
          }}
        />
      </Card>

      <Modal
        title={t('purchases.update_status')}
        open={transitionModalOpen}
        onOk={handleTransition}
        onCancel={() => setTransitionModalOpen(false)}
        confirmLoading={transitionMutation.isPending}
      >
        <Space direction="vertical" style={{ width: '100%' }}>
          <div>
            <div style={{ marginBottom: 4 }}>{t('purchases.target_status')}</div>
            <Select
              value={targetStatus || undefined}
              onChange={setTargetStatus}
              placeholder={t('purchases.select_status')}
              style={{ width: '100%' }}
            >
              {availableTransitions.map((s) => (
                <Select.Option key={s} value={s}>
                  {t('purchases.status.' + s)}
                </Select.Option>
              ))}
            </Select>
          </div>
          <div>
            <div style={{ marginBottom: 4 }}>{t('purchases.notes')}</div>
            <Input.TextArea
              value={transitionNotes}
              onChange={(e) => setTransitionNotes(e.target.value)}
              rows={3}
            />
          </div>
        </Space>
      </Modal>
    </div>
  );
}
