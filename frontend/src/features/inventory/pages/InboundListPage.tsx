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
} from 'antd';
import { PlusOutlined, SearchOutlined, InboxOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { DataTable } from '@/shared/components/DataTable';
import { usePagination } from '@/shared/hooks/usePagination';
import { INBOUND_TYPES, PIPE_TYPES } from '@/shared/constants';
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

interface InboundPipeFormRow {
  pipe_type?: string;
  pipe_id?: number;
}

function parsePipeIds(input: string): number[] {
  const tokens = input
    .replace(/[，、；;\r\n\t]+/g, ',')
    .replace(/\s+/g, ',')
    .split(',')
    .map((token) => token.trim())
    .filter(Boolean);

  const ids: number[] = [];
  for (const token of tokens) {
    const rangeMatch = token.match(/^(\d+)\s*-\s*(\d+)$/);
    if (rangeMatch) {
      const start = Number(rangeMatch[1]);
      const end = Number(rangeMatch[2]);
      if (Number.isInteger(start) && Number.isInteger(end) && start > 0 && end >= start) {
        for (let id = start; id <= end; id += 1) {
          ids.push(id);
        }
      }
      continue;
    }

    const id = Number(token);
    if (Number.isInteger(id) && id > 0) {
      ids.push(id);
    }
  }

  return [...new Set(ids)];
}

function pipeRowKey(pipeType: string | undefined, pipeId: number | undefined): string | null {
  return pipeType && typeof pipeId === 'number' ? `${pipeType}:${pipeId}` : null;
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
  const [batchModalOpen, setBatchModalOpen] = useState(false);
  const [batchPipeType, setBatchPipeType] = useState('casing');
  const [batchPipeIds, setBatchPipeIds] = useState('');
  const [form] = Form.useForm<CreateInboundData>();
  const [rejectForm] = Form.useForm<{ reason: string }>();

  const handlePOSelect = useCallback(
    (po: PurchaseOrder) => {
      const pipes: { pipe_type: string; pipe_id?: number }[] = [];
      form.setFieldsValue({
        inbound_type: 'purchase',
        order_id: po.id,
        supplier_id: po.supplier_id,
        notes: t('inbound.from_po_template', { order_no: po.order_no }),
        pipes: pipes.length > 0 ? pipes : [{ pipe_type: 'casing', pipe_id: undefined }],
      });
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
    form.setFieldsValue({ pipes: [{ pipe_type: 'casing', pipe_id: undefined }] });
    setModalOpen(true);
  };

  const handleCreate = async () => {
    try {
      const values = await form.validateFields();
      const payload = {
        ...values,
        order_id: values.order_id != null ? Number(values.order_id) : undefined,
        supplier_id:
          values.supplier_id != null ? Number(values.supplier_id) : undefined,
        pipes: (values.pipes ?? []).map((pipe) => ({
          ...pipe,
          pipe_id: Number(pipe.pipe_id),
        })),
      };
      await createMutation.mutateAsync(payload);
      message.success(t('common.operate_success'));
      setModalOpen(false);
    } catch (err) {
      console.error('create inbound failed', err);
      message.error(t('common.operate_failed'));
    }
  };

  const handleBatchAddPipes = () => {
    const ids = parsePipeIds(batchPipeIds);
    if (ids.length === 0) {
      message.error(t('common.required'));
      return;
    }

    const currentPipes = (form.getFieldValue('pipes') || []) as InboundPipeFormRow[];
    const existingKeys = new Set(
      currentPipes
        .map((pipe) => pipeRowKey(pipe.pipe_type, pipe.pipe_id))
        .filter((key): key is string => key !== null),
    );
    const rowsToAdd = ids
      .filter((pipeId) => !existingKeys.has(pipeRowKey(batchPipeType, pipeId) ?? ''))
      .map((pipeId) => ({ pipe_type: batchPipeType, pipe_id: pipeId }));

    if (rowsToAdd.length === 0) {
      message.warning(t('inbound.pipe_already_added', 'This pipe has already been added to the list'));
      return;
    }

    form.setFieldsValue({ pipes: [...currentPipes, ...rowsToAdd] });
    setBatchPipeIds('');
    setBatchModalOpen(false);
    message.success(t('common.operate_success'));
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
          <Form.Item label={t('inbound.pipes')}>
            <Form.List name="pipes" initialValue={[{ pipe_type: 'casing' }]}>
              {(fields, { add, remove }) => (
                <>
                  {fields.map(({ key, name, ...rest }) => (
                    <div key={key} style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
                      <Form.Item
                        {...rest}
                        name={[name, 'pipe_type']}
                        rules={[{ required: true }]}
                        noStyle
                      >
                        <Select style={{ width: 140 }}>
                          {PIPE_TYPES.map((pt) => (
                            <Select.Option key={pt} value={pt}>
                              {t('pipe_type.' + pt)}
                            </Select.Option>
                          ))}
                        </Select>
                      </Form.Item>
                      <Form.Item
                        {...rest}
                        name={[name, 'pipe_id']}
                        rules={[{ required: true, message: t('common.required') }]}
                        noStyle
                      >
                        <InputNumber
                          placeholder={t('inbound.pipe_id_placeholder')}
                          min={1}
                          style={{ width: 120 }}
                        />
                      </Form.Item>
                      {fields.length > 1 && (
                        <Button size="small" danger onClick={() => remove(name)}>
                          {t('common.delete')}
                        </Button>
                      )}
                    </div>
                  ))}
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 8, width: '100%' }}>
                    <Button type="dashed" onClick={() => add({ pipe_type: 'casing' })} block>
                      + {t('inbound.add_pipe')}
                    </Button>
                    <Button
                      type="dashed"
                      onClick={() => {
                        setBatchPipeIds('');
                        setBatchModalOpen(true);
                      }}
                      block
                    >
                      {t('inbound.batch_add_pipes', '批量添加管材')}
                    </Button>
                  </div>
                </>
              )}
            </Form.List>
          </Form.Item>
        </Form>
      </Modal>

      <Modal
        title={t('inbound.batch_add_pipes', '批量添加管材')}
        open={batchModalOpen}
        onOk={handleBatchAddPipes}
        onCancel={() => setBatchModalOpen(false)}
        destroyOnClose
      >
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12, width: '100%' }}>
          <Select
            value={batchPipeType}
            onChange={setBatchPipeType}
            style={{ width: 200 }}
          >
            {PIPE_TYPES.map((type) => (
              <Select.Option key={type} value={type}>
                {t(`pipe_type.${type}`, type)}
              </Select.Option>
            ))}
          </Select>
          <Input.TextArea
            rows={6}
            value={batchPipeIds}
            onChange={(event) => setBatchPipeIds(event.target.value)}
            placeholder={t('inbound.batch_pipe_ids_placeholder', '例如：1001,1002,1003 或 1001-1010；支持空格、换行、逗号分隔')}
          />
        </div>
      </Modal>

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
