// 入库管理页 — 使用 DataTable + PageLayout + usePagination
import { useState, useCallback, useMemo } from 'react';
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
import { PlusOutlined, SearchOutlined, InboxOutlined, DeleteOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { PageLayout, ItemPicker } from '@/shared/components';
import type { ItemOption } from '@/shared/components';
import { DataTable } from '@/shared/components/DataTable';
import { usePagination } from '@/shared/hooks/usePagination';
import { INBOUND_TYPES } from '@/shared/constants';
import {
  useInboundRecords,
  useCreateInbound,
  useApproveInbound,
  useRejectInbound,
  useDeleteInbound,
} from '../hooks/useInventory';
import type { InboundRecord, CreateInboundData } from '../api/inventoryApi';
import type { PurchaseOrder } from '../../purchases/types';
import PurchaseOrderSelector from '../components/PurchaseOrderSelector';

const STATUS_COLOR_MAP: Record<string, string> = {
  pending: 'orange',
  auto_approved: 'green',
  approved: 'green',
  rejected: 'red',
};

const TYPE_LABEL_MAP: Record<string, string> = {
  purchase: 'inbound.type.purchase',
  production: 'inbound.type.production',
  return: 'inbound.type.return',
};

interface RowItem {
  item_id: number;
  sku?: string;
  name?: string;
  quantity: number;
}

export default function InboundListPage() {
  const { t } = useTranslation();
  const { page, pageSize, onPaginationChange, reset } = usePagination();
  const [searchText, setSearchText] = useState('');
  const [typeFilter, setTypeFilter] = useState<string | undefined>();
  const [statusFilter, setStatusFilter] = useState<string | undefined>();
  const [modalOpen, setModalOpen] = useState(false);
  const [rejectModalOpen, setRejectModalOpen] = useState(false);
  const [rejectTargetId, setRejectTargetId] = useState<number | null>(null);
  const [poSelectorOpen, setPoSelectorOpen] = useState(false);
  const [itemModalOpen, setItemModalOpen] = useState(false);
  const [items, setItems] = useState<RowItem[]>([]);
  const [form] = Form.useForm<CreateInboundData>();
  const [rejectForm] = Form.useForm<{ reason: string }>();

  const handlePOSelect = useCallback(
    (po: PurchaseOrder) => {
      form.setFieldsValue({
        inbound_type: 'purchase',
        order_id: po.id,
        supplier_id: po.supplier_id,
        notes: t('inbound.from_po_template', { order_no: po.order_no }),
      });
      setItems([]);
      setPoSelectorOpen(false);
      message.success(t('inbound.template_applied'));
    },
    [form, t],
  );

  const { data, isLoading } = useInboundRecords({
    page,
    page_size: pageSize,
    q: searchText || undefined,
    inbound_type: typeFilter,
    approval_status: statusFilter,
  });

  const createMutation = useCreateInbound();
  const approveMutation = useApproveInbound();
  const rejectMutation = useRejectInbound();
  const deleteMutation = useDeleteInbound();

  const openCreateModal = () => {
    form.resetFields();
    setItems([]);
    setModalOpen(true);
  };

  const handleCreate = async () => {
    try {
      const values = await form.validateFields();
      const payload: CreateInboundData = {
        ...values,
        order_id: values.order_id != null ? Number(values.order_id) : undefined,
        supplier_id:
          values.supplier_id != null ? Number(values.supplier_id) : undefined,
        items: items.map((it) => ({ item_id: it.item_id, quantity: it.quantity })),
      };
      if (items.length === 0) {
        message.error(t('common.required'));
        return;
      }
      await createMutation.mutateAsync(payload);
      message.success(t('common.operate_success'));
      setModalOpen(false);
    } catch (err) {
      console.error('create inbound failed', err);
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
      console.error('create inbound failed', err);
    }
  };

  const columns = useMemo(() => [
    {
      title: t('inbound.inbound_no'),
      dataIndex: 'inbound_no',
      key: 'inbound_no',
    },
    {
      title: t('inbound.inbound_type'),
      dataIndex: 'inbound_type',
      key: 'inbound_type',
      render: (type: string) => <Tag>{t(TYPE_LABEL_MAP[type] ?? type)}</Tag>,
    },
    {
      title: t('inbound.approval_status'),
      dataIndex: 'approval_status',
      key: 'approval_status',
      render: (status: string) => (
        <Tag color={STATUS_COLOR_MAP[status] ?? 'default'}>
          {t(`inbound.status.${status}`)}
        </Tag>
      ),
    },
    {
      title: t('inbound.notes'),
      dataIndex: 'notes',
      key: 'notes',
      render: (notes: string | undefined) => notes || '-',
    },
    {
      title: t('inbound.created_at'),
      dataIndex: 'created_at',
      key: 'created_at',
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: InboundRecord) => (
        <>
          {record.approval_status === 'pending' && (
            <>
              <Button
                type="link"
                size="small"
                onClick={() => handleApprove(record.id)}
                loading={approveMutation.isPending}
              >
                {t('inbound.approve')}
              </Button>
              <Button
                type="link"
                size="small"
                danger
                onClick={() => openRejectModal(record.id)}
              >
                {t('inbound.reject')}
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
      title={t('inbound.title')}
      extra={
        <Button type="primary" icon={<PlusOutlined />} onClick={openCreateModal}>
          {t('inbound.create_inbound')}
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
          placeholder={t('inbound.inbound_type')}
          allowClear
          style={{ width: 130 }}
          value={typeFilter}
          onChange={(v) => {
            setTypeFilter(v);
            reset();
          }}
          options={INBOUND_TYPES.map((t2) => ({
            label: t(TYPE_LABEL_MAP[t2]),
            value: t2,
          }))}
        />
        <Select
          placeholder={t('inbound.approval_status')}
          allowClear
          style={{ width: 130 }}
          value={statusFilter}
          onChange={(v) => {
            setStatusFilter(v);
            reset();
          }}
          options={[
            { label: t('inbound.status.pending'), value: 'pending' },
            { label: t('inbound.status.approved'), value: 'approved' },
            { label: t('inbound.status.rejected'), value: 'rejected' },
          ]}
        />
      </div>

      <DataTable<InboundRecord>
        columns={columns}
        items={data?.items}
        total={data?.total}
        page={page}
        pageSize={pageSize}
        loading={isLoading}
        onPaginationChange={onPaginationChange}
      />

      <Modal
        title={t('inbound.create_inbound')}
        open={modalOpen}
        onOk={handleCreate}
        onCancel={() => setModalOpen(false)}
        confirmLoading={createMutation.isPending}
        destroyOnClose
        width={600}
      >
        <Form form={form} layout="vertical" style={{ marginTop: 16 }}>
          <div style={{ marginBottom: 16 }}>
            <Button
              icon={<InboxOutlined />}
              onClick={() => setPoSelectorOpen(true)}
              block
            >
              {t('inbound.from_purchase_order')}
            </Button>
          </div>

          <Form.Item
            name="inbound_type"
            label={t('inbound.inbound_type')}
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Select
              options={INBOUND_TYPES.map((t2) => ({
                label: t(TYPE_LABEL_MAP[t2]),
                value: t2,
              }))}
            />
          </Form.Item>
          <Form.Item name="order_id" label={t('inbound.order_id')}>
            <InputNumber style={{ width: '100%' }} />
          </Form.Item>
          <Form.Item name="supplier_id" label={t('inbound.supplier_id')}>
            <InputNumber style={{ width: '100%' }} />
          </Form.Item>
          <Form.Item name="notes" label={t('inbound.notes')}>
            <Input.TextArea rows={3} />
          </Form.Item>
          <div style={{ marginBottom: 8, fontWeight: 500 }}>{t('inbound.items', '入库商品')}</div>
          <Table<RowItem>
            rowKey={(_, index) => String(index)}
            size="small"
            pagination={false}
            dataSource={items}
            columns={[
              {
                title: t('inbound.item', '商品'),
                key: 'item',
                render: (_: unknown, record: RowItem) =>
                  record.name ? `${record.sku ?? ''} — ${record.name}` : record.sku || `#${record.item_id}`,
              },
              {
                title: t('inbound.quantity', '数量'),
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
                {t('inbound.add_item', '添加商品')}
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

      <PurchaseOrderSelector
        open={poSelectorOpen}
        onCancel={() => setPoSelectorOpen(false)}
        onSelect={handlePOSelect}
      />

      <Modal
        title={t('inbound.reject')}
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
            label={t('inbound.reject_reason')}
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Input.TextArea rows={3} />
          </Form.Item>
        </Form>
      </Modal>
    </PageLayout>
  );
}
