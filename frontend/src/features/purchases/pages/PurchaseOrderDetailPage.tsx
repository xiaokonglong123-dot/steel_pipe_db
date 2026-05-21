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
      title: 'Pipe Type',
      dataIndex: 'pipe_type',
      key: 'pipe_type',
    },
    {
      title: 'Grade',
      dataIndex: 'grade',
      key: 'grade',
      render: (grade: string) => <Tag color="blue">{grade}</Tag>,
    },
    {
      title: 'OD (in)',
      dataIndex: 'od',
      key: 'od',
    },
    {
      title: 'WT (in)',
      dataIndex: 'wt',
      key: 'wt',
    },
    {
      title: 'Length (m)',
      dataIndex: 'length',
      key: 'length',
      render: (val: number | undefined) => val ?? '-',
    },
    {
      title: 'Quantity',
      dataIndex: 'quantity',
      key: 'quantity',
    },
    {
      title: 'Unit Price',
      dataIndex: 'unit_price',
      key: 'unit_price',
      render: (val: number) => `$${val.toFixed(2)}`,
    },
    {
      title: 'Total Price',
      dataIndex: 'total_price',
      key: 'total_price',
      render: (val: number) => `$${val.toFixed(2)}`,
    },
    {
      title: 'Notes',
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
        <h2 style={{ margin: 0 }}>Purchase Order — {order.order_number}</h2>
        <Space>
          {showTransitionBtn && (
            <Button
              type="primary"
              onClick={() => setTransitionModalOpen(true)}
            >
              Update Status
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

      <Card title="Order Info" style={{ marginBottom: 24 }}>
        <Descriptions bordered column={{ xs: 1, sm: 2, lg: 3 }}>
          <Descriptions.Item label="Order Number">{order.order_number}</Descriptions.Item>
          <Descriptions.Item label="Supplier Name">{order.supplier_name}</Descriptions.Item>
          <Descriptions.Item label="Order Date">{order.order_date}</Descriptions.Item>
          <Descriptions.Item label="Expected Date">{order.expected_date ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Status">
            <Tag color={STATUS_COLORS[order.status] ?? 'default'}>{order.status}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="Total Amount">${order.total_amount.toFixed(2)}</Descriptions.Item>
          <Descriptions.Item label="Notes" span={3}>
            {order.notes ?? '-'}
          </Descriptions.Item>
        </Descriptions>
      </Card>

      <Card title="Items">
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
                  <strong>Total</strong>
                </Table.Summary.Cell>
                <Table.Summary.Cell index={1}>
                  <strong>${total.toFixed(2)}</strong>
                </Table.Summary.Cell>
                <Table.Summary.Cell index={2} />
              </Table.Summary.Row>
            );
          }}
        />
      </Card>

      <Modal
        title="Update Status"
        open={transitionModalOpen}
        onOk={handleTransition}
        onCancel={() => setTransitionModalOpen(false)}
        confirmLoading={transitionMutation.isPending}
      >
        <Space direction="vertical" style={{ width: '100%' }}>
          <div>
            <div style={{ marginBottom: 4 }}>Target Status</div>
            <Select
              value={targetStatus || undefined}
              onChange={setTargetStatus}
              placeholder="Select status"
              style={{ width: '100%' }}
            >
              {availableTransitions.map((s) => (
                <Select.Option key={s} value={s}>
                  {s}
                </Select.Option>
              ))}
            </Select>
          </div>
          <div>
            <div style={{ marginBottom: 4 }}>Notes</div>
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
