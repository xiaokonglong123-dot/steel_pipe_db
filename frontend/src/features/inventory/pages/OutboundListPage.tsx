// 出库管理页 — 使用 DataTable + PageLayout + usePagination
import { useState, useMemo } from 'react';
import {
  Button,
  Tag,
  Input,
  InputNumber,
  Modal,
  Form,
  Select,
  Popconfirm,
  message,
  Table,
} from 'antd';
import { PlusOutlined, SearchOutlined, DeleteOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { PageLayout, ItemPicker } from '@/shared/components';
import type { ItemOption } from '@/shared/components';
import { DataTable } from '@/shared/components/DataTable';
import { usePagination } from '@/shared/hooks/usePagination';
import { OUTBOUND_TYPES } from '@/shared/constants';
import {
  useOutboundRecords,
  useCreateOutbound,
  useApproveOutbound,
  useRejectOutbound,
  useDeleteOutbound,
} from '../hooks/useInventory';
import type { OutboundRecord, CreateOutboundData } from '../api/inventoryApi';

const STATUS_COLOR_MAP: Record<string, string> = {
  pending: 'orange',
  auto_approved: 'green',
  approved: 'green',
  rejected: 'red',
};

const TYPE_LABEL_MAP: Record<string, string> = {
  sales: 'outbound.type.sales',
  transfer: 'outbound.type.transfer',
  scrapped: 'outbound.type.scrapped',
};

interface RowItem {
  item_id: number;
  sku?: string;
  name?: string;
  quantity: number;
}

export default function OutboundListPage() {
  const { t } = useTranslation();
  const { page, pageSize, onPaginationChange, reset } = usePagination();
  const [searchText, setSearchText] = useState('');
  const [typeFilter, setTypeFilter] = useState<string | undefined>();
  const [statusFilter, setStatusFilter] = useState<string | undefined>();
  const [modalOpen, setModalOpen] = useState(false);
  const [rejectModalOpen, setRejectModalOpen] = useState(false);
  const [rejectTargetId, setRejectTargetId] = useState<number | null>(null);
  const [itemModalOpen, setItemModalOpen] = useState(false);
  const [items, setItems] = useState<RowItem[]>([]);
  const [form] = Form.useForm<CreateOutboundData>();
  const [rejectForm] = Form.useForm<{ reason: string }>();

  const { data, isLoading } = useOutboundRecords({
    page,
    page_size: pageSize,
    q: searchText || undefined,
    outbound_type: typeFilter,
    approval_status: statusFilter,
  });

  const createMutation = useCreateOutbound();
  const approveMutation = useApproveOutbound();
  const rejectMutation = useRejectOutbound();
  const deleteMutation = useDeleteOutbound();

  const openCreateModal = () => {
    form.resetFields();
    setItems([]);
    setModalOpen(true);
  };

  const handleCreate = async () => {
    try {
      const values = await form.validateFields();
      if (items.length === 0) {
        message.error(t('common.required'));
        return;
      }
      const payload: CreateOutboundData = {
        ...values,
        order_id: values.order_id != null ? Number(values.order_id) : undefined,
        customer_id:
          values.customer_id != null ? Number(values.customer_id) : undefined,
        items: items.map((it) => ({ item_id: it.item_id, quantity: it.quantity })),
      };
      await createMutation.mutateAsync(payload);
      message.success(t('common.operate_success'));
      setModalOpen(false);
    } catch (err) {
      console.error('create outbound failed', err);
      message.error(t('common.operate_failed'));
    }
  };

  const handleApprove = (id: number) => {
    approveMutation.mutate(
      { id },
      {
        onSuccess: () => message.success(t('common.operate_success')),
        onError: () => message.error(t('common.operate_failed')),
      },
    );
  };

  const openRejectModal = (id: number) => {
    setRejectTargetId(id);
    rejectForm.resetFields();
    setRejectModalOpen(true);
  };

  const handleReject = async () => {
    if (rejectTargetId === null) return;
    try {
      const values = await rejectForm.validateFields();
      await rejectMutation.mutateAsync({ id: rejectTargetId, reason: values.reason });
      message.success(t('common.operate_success'));
      setRejectModalOpen(false);
      setRejectTargetId(null);
    } catch (err) {
      console.error('create outbound failed', err);
    }
  };

  const columns = useMemo(() => [
    {
      title: t('outbound.outbound_no'),
      dataIndex: 'outbound_no',
      key: 'outbound_no',
    },
    {
      title: t('outbound.outbound_type'),
      dataIndex: 'outbound_type',
      key: 'outbound_type',
      render: (type: string) => <Tag>{t(TYPE_LABEL_MAP[type] ?? type)}</Tag>,
    },
    {
      title: t('outbound.approval_status'),
      dataIndex: 'approval_status',
      key: 'approval_status',
      render: (status: string) => (
        <Tag color={STATUS_COLOR_MAP[status] ?? 'default'}>
          {t(`outbound.status.${status}`)}
        </Tag>
      ),
    },
    {
      title: t('outbound.notes'),
      dataIndex: 'notes',
      key: 'notes',
      render: (notes: string | undefined) => notes || '-',
    },
    {
      title: t('outbound.created_at'),
      dataIndex: 'created_at',
      key: 'created_at',
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: OutboundRecord) => (
        <>
          {record.approval_status === 'pending' && (
            <>
              <Button
                type="link"
                size="small"
                onClick={() => handleApprove(record.id)}
                loading={approveMutation.isPending}
              >
                {t('outbound.approve')}
              </Button>
              <Button
                type="link"
                size="small"
                danger
                onClick={() => openRejectModal(record.id)}
              >
                {t('outbound.reject')}
              </Button>
            </>
          )}
          <Popconfirm
            title={t('common.confirm_delete')}
            onConfirm={() => deleteMutation.mutate(record.id)}
          >
            <Button type="link" danger size="small">
              {t('common.delete')}
            </Button>
          </Popconfirm>
        </>
      ),
    },
  ], [t, handleApprove, approveMutation, openRejectModal, deleteMutation]);

  return (
    <PageLayout
      title={t('outbound.title')}
      extra={
        <Button type="primary" icon={<PlusOutlined />} onClick={openCreateModal}>
          {t('outbound.create_outbound')}
        </Button>
      }
    >
      <div style={{ display: 'flex', gap: 12, marginBottom: 16, flexWrap: 'wrap' }}>
        <Input
          placeholder={t('common.search')}
          prefix={<SearchOutlined />}
          value={searchText}
          onChange={(e) => {
            setSearchText(e.target.value);
            reset();
          }}
          style={{ width: 200 }}
        />
        <Select
          placeholder={t('outbound.outbound_type')}
          allowClear
          style={{ width: 130 }}
          value={typeFilter}
          onChange={(v) => {
            setTypeFilter(v);
            reset();
          }}
          options={OUTBOUND_TYPES.map((ot) => ({
            label: t(TYPE_LABEL_MAP[ot]),
            value: ot,
          }))}
        />
        <Select
          placeholder={t('outbound.approval_status')}
          allowClear
          style={{ width: 130 }}
          value={statusFilter}
          onChange={(v) => {
            setStatusFilter(v);
            reset();
          }}
          options={[
            { label: t('outbound.status.pending'), value: 'pending' },
            { label: t('outbound.status.approved'), value: 'approved' },
            { label: t('outbound.status.rejected'), value: 'rejected' },
          ]}
        />
      </div>

      <DataTable<OutboundRecord>
        columns={columns}
        items={data?.items}
        total={data?.total}
        page={page}
        pageSize={pageSize}
        loading={isLoading}
        onPaginationChange={onPaginationChange}
      />

      <Modal
        title={t('outbound.create_outbound')}
        open={modalOpen}
        onOk={handleCreate}
        onCancel={() => setModalOpen(false)}
        confirmLoading={createMutation.isPending}
        destroyOnClose
        width={600}
      >
        <Form form={form} layout="vertical" style={{ marginTop: 16 }}>
          <Form.Item
            name="outbound_type"
            label={t('outbound.outbound_type')}
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Select
              options={OUTBOUND_TYPES.map((ot) => ({
                label: t(TYPE_LABEL_MAP[ot]),
                value: ot,
              }))}
            />
          </Form.Item>
          <Form.Item name="order_id" label={t('outbound.order_id')}>
            <InputNumber style={{ width: '100%' }} />
          </Form.Item>
          <Form.Item name="customer_id" label={t('outbound.customer_id')}>
            <InputNumber style={{ width: '100%' }} />
          </Form.Item>
          <Form.Item name="notes" label={t('outbound.notes')}>
            <Input.TextArea rows={3} />
          </Form.Item>
          <div style={{ marginBottom: 8, fontWeight: 500 }}>{t('outbound.items', '出库商品')}</div>
          <Table<RowItem>
            rowKey={(_, index) => String(index)}
            size="small"
            pagination={false}
            dataSource={items}
            columns={[
              {
                title: t('outbound.item', '商品'),
                key: 'item',
                render: (_: unknown, record: RowItem) =>
                  record.name ? `${record.sku ?? ''} — ${record.name}` : record.sku || `#${record.item_id}`,
              },
              {
                title: t('outbound.quantity', '数量'),
                dataIndex: 'quantity',
                key: 'quantity',
                width: 110,
                render: (_: unknown, record: RowItem, index: number) => (
                  <InputNumber
                    min={0}
                    step={1}
                    value={record.quantity}
                    style={{ width: '100%' }}
                    onChange={(v) =>
                      setItems((prev) =>
                        prev.map((it, i) => (i === index ? { ...it, quantity: v ?? 0 } : it)),
                      )
                    }
                  />
                ),
              },
              {
                key: 'actions',
                width: 50,
                render: (_: unknown, __: unknown, index: number) => (
                  <Button
                    type="link"
                    danger
                    size="small"
                    icon={<DeleteOutlined />}
                    onClick={() => setItems((prev) => prev.filter((_, i) => i !== index))}
                  />
                ),
              },
            ]}
            footer={() => (
              <Button
                type="dashed"
                icon={<PlusOutlined />}
                onClick={() => setItemModalOpen(true)}
                block
              >
                {t('outbound.add_item', '添加商品')}
              </Button>
            )}
          />
        </Form>
      </Modal>

      <ItemPicker
        open={itemModalOpen}
        onCancel={() => setItemModalOpen(false)}
        onSelect={(picked: ItemOption[]) => {
          const additions: RowItem[] = picked.map((it) => ({
            item_id: it.id,
            sku: it.sku,
            name: it.name,
            quantity: 1,
          }));
          setItems((prev) => [...prev, ...additions]);
          setItemModalOpen(false);
        }}
      />

      <Modal
        title={t('outbound.reject')}
        open={rejectModalOpen}
        onOk={handleReject}
        onCancel={() => {
          setRejectModalOpen(false);
          setRejectTargetId(null);
        }}
        confirmLoading={rejectMutation.isPending}
        destroyOnClose
      >
        <Form form={rejectForm} layout="vertical" style={{ marginTop: 16 }}>
          <Form.Item
            name="reason"
            label={t('outbound.reject_reason')}
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Input.TextArea rows={3} />
          </Form.Item>
        </Form>
      </Modal>
    </PageLayout>
  );
}
