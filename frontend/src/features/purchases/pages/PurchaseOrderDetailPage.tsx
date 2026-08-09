// 采购订单详情页 — 使用 PageLayout 共享组件
import { useState } from 'react';
import { Button, Descriptions, Tag, Card, Table, Select, message, Modal, Input, InputNumber } from 'antd';
import { EditOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { usePurchase, useTransitionPurchaseOrder, useApprovePurchaseOrder, useRejectPurchaseOrder, useLinkInbound } from '../hooks/usePurchases';

const STATUS_COLORS: Record<string, string> = {
  draft: 'default',
  pending: 'orange',
  approved: 'blue',
  rejected: 'red',
  completed: 'green',
  cancelled: 'red',
};

const STATUS_TRANSITIONS: Record<string, string[]> = {
  draft: ['pending', 'cancelled'],
  pending: ['approved', 'cancelled'],
  approved: ['completed', 'cancelled'],
};

export default function PurchaseOrderDetailPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const orderId = Number(id);

  const { data: detail, isLoading } = usePurchase(orderId);
  const order = detail?.order;
  const items = detail?.items ?? [];
  const transitionMutation = useTransitionPurchaseOrder(orderId);
  const approveMutation = useApprovePurchaseOrder(orderId);
  const rejectMutation = useRejectPurchaseOrder(orderId);
  const linkMutation = useLinkInbound(orderId);

  const [transitionModalOpen, setTransitionModalOpen] = useState(false);
  const [targetStatus, setTargetStatus] = useState<string>('');
  const [transitionNotes, setTransitionNotes] = useState('');

  const [rejectModalOpen, setRejectModalOpen] = useState(false);
  const [rejectReason, setRejectReason] = useState('');
  const [linkModalOpen, setLinkModalOpen] = useState(false);
  const [inboundRecordId, setInboundRecordId] = useState<number | undefined>();

  const handleApprove = async () => {
    try {
      await approveMutation.mutateAsync({});
      message.success(t('common.operate_success'));
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  const handleReject = async () => {
    try {
      await rejectMutation.mutateAsync({ reason: rejectReason });
      message.success(t('common.operate_success'));
      setRejectModalOpen(false);
      setRejectReason('');
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  const handleLink = async () => {
    if (!inboundRecordId) return;
    try {
      await linkMutation.mutateAsync(inboundRecordId);
      message.success(t('common.operate_success'));
      setLinkModalOpen(false);
      setInboundRecordId(undefined);
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  const handleTransition = async () => {
    try {
      await transitionMutation.mutateAsync({
        status: targetStatus,
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
      title: t('purchases.item', '商品'),
      key: 'item',
      render: (_: unknown, record: { item_id: number; sku?: string; name?: string }) =>
        record.name ? `${record.sku ?? ''} — ${record.name}` : record.sku || `#${record.item_id}`,
    },
    {
      title: t('purchases.quantity'),
      dataIndex: 'quantity',
      key: 'quantity',
    },
    {
      title: t('purchases.received_quantity'),
      dataIndex: 'received_quantity',
      key: 'received_quantity',
    },
    {
      title: t('purchases.unit_price'),
      dataIndex: 'unit_price',
      key: 'unit_price',
      render: (val: number | null) => val != null ? `¥${val.toFixed(2)}` : '-',
    },
    {
      title: t('purchases.total_price'),
      dataIndex: 'total_price',
      key: 'total_price',
      render: (val: number | null) => val != null ? `¥${val.toFixed(2)}` : '-',
    },
    {
      title: t('purchases.notes'),
      dataIndex: 'notes',
      key: 'notes',
      render: (val: string | undefined) => val ?? '-',
    },
  ];

  return (
    <PageLayout
      title={`${t('purchases.purchase_order')} — ${order.order_no}`}
      onBack={() => navigate('/purchases')}
      extra={
        <>
          {order.status === 'pending' && (
            <>
              <Button type="primary" onClick={handleApprove} loading={approveMutation.isPending}>
                {t('purchases.approve')}
              </Button>
              <Button danger onClick={() => setRejectModalOpen(true)}>
                {t('purchases.reject')}
              </Button>
            </>
          )}
          {(order.status === 'approved' || order.status === 'completed') && (
            <Button onClick={() => setLinkModalOpen(true)}>
              {t('purchases.link_inbound')}
            </Button>
          )}
          {showTransitionBtn && (
            <Button type="primary" onClick={() => setTransitionModalOpen(true)}>
              {t('purchases.update_status')}
            </Button>
          )}
          <Button
            icon={<EditOutlined />}
            onClick={() => navigate(`/purchases/${order.id}/edit`)}
          >
            {t('common.edit')}
          </Button>
        </>
      }
    >
      <Card title={t('purchases.order_info')} style={{ marginBottom: 24 }}>
        <Descriptions bordered column={{ xs: 1, sm: 2, lg: 3 }}>
          <Descriptions.Item label={t('purchases.order_number')}>{order.order_no}</Descriptions.Item>
          <Descriptions.Item label={t('purchases.order_date')}>{order.order_date}</Descriptions.Item>
          <Descriptions.Item label={t('purchases.status')}>
            <Tag color={STATUS_COLORS[order.status] ?? 'default'}>{t(`purchases.status.${order.status}`)}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label={t('purchases.total_amount')}>¥{order.total_amount?.toFixed(2) ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('purchases.notes')} span={3}>
            {order.notes ?? '-'}
          </Descriptions.Item>
        </Descriptions>
      </Card>

      <Card title={t('purchases.items')}>
        <Table
          columns={itemColumns}
          dataSource={items}
          rowKey="id"
          pagination={false}
          summary={() => {
            const total = items.reduce(
              (sum, item) => sum + (item.total_price ?? 0),
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
        title={t('purchases.reject')}
        open={rejectModalOpen}
        onOk={handleReject}
        onCancel={() => { setRejectModalOpen(false); setRejectReason(''); }}
        confirmLoading={rejectMutation.isPending}
      >
        <Input.TextArea
          value={rejectReason}
          onChange={(e) => setRejectReason(e.target.value)}
          placeholder={t('purchases.reject_reason')}
          rows={3}
        />
      </Modal>

      <Modal
        title={t('purchases.link_inbound')}
        open={linkModalOpen}
        onOk={handleLink}
        onCancel={() => { setLinkModalOpen(false); setInboundRecordId(undefined); }}
        confirmLoading={linkMutation.isPending}
      >
        <InputNumber
          style={{ width: '100%' }}
          value={inboundRecordId}
          onChange={(val) => setInboundRecordId(val ?? undefined)}
          placeholder={t('purchases.inbound_record_id') || 'Inbound Record ID'}
          min={1}
        />
      </Modal>

      <Modal
        title={t('purchases.update_status')}
        open={transitionModalOpen}
        onOk={handleTransition}
        onCancel={() => setTransitionModalOpen(false)}
        confirmLoading={transitionMutation.isPending}
      >
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12, width: '100%' }}>
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
        </div>
      </Modal>
    </PageLayout>
  );
}
