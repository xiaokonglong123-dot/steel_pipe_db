// 入库管理页 — 入库记录列表 + 弹窗创建/编辑，支持采购/生产/退货/调拨等入库类型
import { useState, useCallback } from 'react';
import {
  Table,
  Button,
  Space,
  Tag,
  Input,
  Modal,
  Form,
  Select,
  Popconfirm,
  message,
} from 'antd';
import { PlusOutlined, SearchOutlined, InboxOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
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

const INBOUND_TYPES = ['purchase', 'production', 'return', 'transfer'];
const PIPE_TYPES = ['casing', 'tubing', 'coupling', 'accessory'];

const STATUS_COLOR_MAP: Record<string, string> = {
  pending: 'orange',
  approved: 'green',
  rejected: 'red',
};

const TYPE_LABEL_MAP: Record<string, string> = {
  purchase: 'inbound.type.purchase',
  production: 'inbound.type.production',
  return: 'inbound.type.return',
  transfer: 'inbound.type.transfer',
};

export default function InboundListPage() {
  const { t } = useTranslation();
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [searchText, setSearchText] = useState('');
  const [typeFilter, setTypeFilter] = useState<string | undefined>();
  const [statusFilter, setStatusFilter] = useState<string | undefined>();
  const [modalOpen, setModalOpen] = useState(false);
  const [rejectModalOpen, setRejectModalOpen] = useState(false);
  const [rejectTargetId, setRejectTargetId] = useState<number | null>(null);
  const [poSelectorOpen, setPoSelectorOpen] = useState(false);
  const [form] = Form.useForm<CreateInboundData>();
  const [rejectForm] = Form.useForm<{ reason: string }>();

  const handlePOSelect = useCallback(
    (po: PurchaseOrder) => {
      const pipes: { pipe_type: string; pipe_id?: number }[] = [];
      for (const item of po.items) {
        for (let i = 0; i < item.quantity; i++) {
          pipes.push({ pipe_type: item.pipe_type, pipe_id: undefined });
        }
      }
      form.setFieldsValue({
        inbound_type: 'purchase',
        order_id: po.id,
        supplier_id: po.supplier_id,
        notes: t('inbound.from_po_template', { order_no: po.order_number }),
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
      await createMutation.mutateAsync(values);
      message.success(t('common.operate_success'));
      setModalOpen(false);
    } catch {
      // validation failed or API error
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
    } catch {
      // validation failed or API error
    }
  };

  const columns = [
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
        <Space>
          {record.approval_status === 'pending' && (
            <>
              <Button
                type="link"
                onClick={() => handleApprove(record.id)}
                loading={approveMutation.isPending}
              >
                {t('inbound.approve')}
              </Button>
              <Button
                type="link"
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
            <Button type="link" danger>
              {t('common.delete')}
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <div>
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          marginBottom: 16,
          flexWrap: 'wrap',
          gap: 8,
        }}
      >
        <Space wrap>
          <Input
            placeholder={t('common.search')}
            prefix={<SearchOutlined />}
            value={searchText}
            onChange={(e) => {
              setSearchText(e.target.value);
              setPage(1);
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
              setPage(1);
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
              setPage(1);
            }}
            options={[
              { label: t('inbound.status.pending'), value: 'pending' },
              { label: t('inbound.status.approved'), value: 'approved' },
              { label: t('inbound.status.rejected'), value: 'rejected' },
            ]}
          />
        </Space>
        <Button type="primary" icon={<PlusOutlined />} onClick={openCreateModal}>
          {t('inbound.create_inbound')}
        </Button>
      </div>

      <Table
        columns={columns}
        dataSource={data?.items}
        rowKey="id"
        loading={isLoading}
        pagination={{
          current: page,
          pageSize,
          total: data?.total,
          onChange: (p, ps) => {
            setPage(p);
            setPageSize(ps);
          },
          showSizeChanger: true,
          showTotal: (total) => t('common.total_items', { total }),
        }}
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
            <Input type="number" />
          </Form.Item>
          <Form.Item name="supplier_id" label={t('inbound.supplier_id')}>
            <Input type="number" />
          </Form.Item>
          <Form.Item name="notes" label={t('inbound.notes')}>
            <Input.TextArea rows={3} />
          </Form.Item>
          <Form.Item label={t('inbound.pipes')}>
            <Form.List name="pipes" initialValue={[{ pipe_type: 'casing' }]}>
              {(fields, { add, remove }) => (
                <>
                  {fields.map(({ key, name, ...rest }) => (
                    <Space key={key} align="baseline" style={{ marginBottom: 8 }}>
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
                        <Input placeholder={t('inbound.pipe_id_placeholder')} type="number" style={{ width: 120 }} />
                      </Form.Item>
                      {fields.length > 1 && (
                        <Button size="small" danger onClick={() => remove(name)}>
                          {t('common.delete')}
                        </Button>
                      )}
                    </Space>
                  ))}
                  <Button type="dashed" onClick={() => add({ pipe_type: 'casing' })} block>
                    + {t('inbound.add_pipe')}
                  </Button>
                </>
              )}
            </Form.List>
          </Form.Item>
        </Form>
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
    </div>
  );
}
