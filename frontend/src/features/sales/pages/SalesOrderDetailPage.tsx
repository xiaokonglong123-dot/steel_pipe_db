// 销售订单详情页 — 使用 PageLayout 共享组件
import { Button, Descriptions, Tag, Card, Table, Select, Input, InputNumber, message, Modal } from 'antd';
import { EditOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useState } from 'react';
import { PageLayout } from '@/shared/components/PageLayout';
import { useSalesOrder, useTransitionSalesOrder, useApproveSalesOrder, useRejectSalesOrder, useLinkOutbound } from '../hooks/useSales';

const STATUS_COLORS: Record<string, string> = {
  draft: 'default',
  pending: 'blue',
  approved: 'cyan',
  rejected: 'red',
  completed: 'green',
  cancelled: 'red',
};

const NEXT_STATUSES: Record<string, string[]> = {
  draft: ['pending', 'cancelled'],
  pending: ['approved', 'cancelled'],
  approved: ['completed', 'cancelled'],
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
  const approveMutation = useApproveSalesOrder(orderId);
  const rejectMutation = useRejectSalesOrder(orderId);
  const linkMutation = useLinkOutbound(orderId);

  const [rejectModalOpen, setRejectModalOpen] = useState(false);
  const [rejectReason, setRejectReason] = useState('');
  const [linkModalOpen, setLinkModalOpen] = useState(false);
  const [outboundRecordId, setOutboundRecordId] = useState<number | undefined>();

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
    if (!outboundRecordId) return;
    try {
      await linkMutation.mutateAsync(outboundRecordId);
      message.success(t('common.operate_success'));
      setLinkModalOpen(false);
      setOutboundRecordId(undefined);
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  const handleTransition = async () => {
    if (!targetStatus) return;
    try {
      await transitionMutation.mutateAsync({ status: targetStatus });
      message.success(t('common.operate_success'));
      setTargetStatus(undefined);
      setTransitionNotes('');
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  const itemColumns = [
    { title: t('pipes.pipe_type'), dataIndex: 'pipe_type', key: 'pipe_type' },
    { title: t('pipes.grade'), dataIndex: 'grade', key: 'grade' },
    { title: t('sales.od'), dataIndex: 'od', key: 'od' },
    { title: t('sales.wt'), dataIndex: 'wt', key: 'wt' },
    { title: t('sales.quantity'), dataIndex: 'quantity', key: 'quantity' },
    { title: t('sales.delivered_quantity'), dataIndex: 'delivered_quantity', key: 'delivered_quantity' },
    { title: t('sales.unit_price'), dataIndex: 'unit_price', key: 'unit_price', render: (v: number | null) => v != null ? v.toLocaleString() : '-' },
    { title: t('sales.total_price'), dataIndex: 'total_price', key: 'total_price', render: (v: number | null) => v != null ? v.toLocaleString() : '-' },
  ];

  if (isLoading) {
    return <div>{t('common.loading')}</div>;
  }

  if (!order) {
    return <div>{t('common.no_data')}</div>;
  }

  const nextStatuses = NEXT_STATUSES[order.status] ?? [];

  return (
    <PageLayout
      title={`${t('sales.sales_order')} — ${order.order_no}`}
      onBack={() => navigate('/sales')}
      extra={
        <>
          {order.status === 'pending' && (
            <>
              <Button type="primary" onClick={handleApprove} loading={approveMutation.isPending}>
                {t('sales.approve')}
              </Button>
              <Button danger onClick={() => setRejectModalOpen(true)}>
                {t('sales.reject')}
              </Button>
            </>
          )}
          {(order.status === 'approved' || order.status === 'completed') && (
            <Button onClick={() => setLinkModalOpen(true)}>
              {t('sales.link_outbound')}
            </Button>
          )}
          <Button icon={<EditOutlined />} onClick={() => navigate(`/sales/${order.id}/edit`)}>
            {t('common.edit')}
          </Button>
        </>
      }
    >
      <Card>
        <Descriptions bordered column={{ xs: 1, sm: 2, lg: 3 }}>
          <Descriptions.Item label={t('sales.order_number')}>{order.order_no}</Descriptions.Item>
          <Descriptions.Item label={t('sales.order_date')}>{order.order_date}</Descriptions.Item>
          <Descriptions.Item label={t('sales.total_amount')}>{order.total_amount?.toLocaleString() ?? '-'}</Descriptions.Item>
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
          <div style={{ display: 'flex', gap: 12, alignItems: 'center' }}>
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
          </div>
        </Card>
      )}

      <Modal
        title={t('sales.reject')}
        open={rejectModalOpen}
        onOk={handleReject}
        onCancel={() => { setRejectModalOpen(false); setRejectReason(''); }}
        confirmLoading={rejectMutation.isPending}
      >
        <Input.TextArea
          value={rejectReason}
          onChange={(e) => setRejectReason(e.target.value)}
          placeholder={t('sales.reject_reason')}
          rows={3}
        />
      </Modal>

      <Modal
        title={t('sales.link_outbound')}
        open={linkModalOpen}
        onOk={handleLink}
        onCancel={() => { setLinkModalOpen(false); setOutboundRecordId(undefined); }}
        confirmLoading={linkMutation.isPending}
      >
        <InputNumber
          style={{ width: '100%' }}
          value={outboundRecordId}
          onChange={(val) => setOutboundRecordId(val ?? undefined)}
          placeholder={t('sales.outbound_record_id') || 'Outbound Record ID'}
          min={1}
        />
      </Modal>
    </PageLayout>
  );
}
