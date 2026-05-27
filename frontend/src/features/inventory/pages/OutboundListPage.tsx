// 出库管理页 — 出库记录列表 + 弹窗创建/编辑，支持销售/报废/调拨等出库类型
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
import { PlusOutlined, SearchOutlined, StockOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import {
  useOutboundRecords,
  useCreateOutbound,
  useApproveOutbound,
  useRejectOutbound,
  useDeleteOutbound,
} from '../hooks/useInventory';
import type { OutboundRecord, CreateOutboundData } from '../api/inventoryApi';
import StockSelector from '../components/StockSelector';

const OUTBOUND_TYPES = ['sales', 'production', 'return', 'transfer', 'scrapped'];
const PIPE_TYPES = ['casing', 'tubing', 'coupling', 'accessory'];

const STATUS_COLOR_MAP: Record<string, string> = {
  pending: 'orange',
  approved: 'green',
  rejected: 'red',
};

const TYPE_LABEL_MAP: Record<string, string> = {
  sales: 'outbound.type.sales',
  production: 'outbound.type.production',
  return: 'outbound.type.return',
  transfer: 'outbound.type.transfer',
  scrapped: 'outbound.type.scrapped',
};

export default function OutboundListPage() {
  const { t } = useTranslation();
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [searchText, setSearchText] = useState('');
  const [typeFilter, setTypeFilter] = useState<string | undefined>();
  const [statusFilter, setStatusFilter] = useState<string | undefined>();
  const [modalOpen, setModalOpen] = useState(false);
  const [rejectModalOpen, setRejectModalOpen] = useState(false);
  const [rejectTargetId, setRejectTargetId] = useState<number | null>(null);
  const [stockSelectorOpen, setStockSelectorOpen] = useState(false);
  const [form] = Form.useForm<CreateOutboundData>();
  const [rejectForm] = Form.useForm<{ reason: string }>();

  const handleStockSelect = useCallback(
    (selectedPipes: { pipe_type: string; pipe_id: number }[]) => {
      const currentPipes = form.getFieldValue('pipes') || [];
      // Merge: append selected pipes to existing, avoiding duplicates by pipe_id
      const existingIds = new Set(currentPipes.map((p: { pipe_id: number }) => p.pipe_id));
      const newPipes = selectedPipes.filter((p) => !existingIds.has(p.pipe_id));
      form.setFieldsValue({ pipes: [...currentPipes, ...newPipes] });
      setStockSelectorOpen(false);
      message.success(t('outbound.stock_added', { count: newPipes.length }));
    },
    [form, t],
  );

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
    form.setFieldsValue({ pipes: [{ pipe_type: 'casing', pipe_id: undefined }] });
    setModalOpen(true);
  };

  const handleCreate = async () => {
    try {
      const values = await form.validateFields();
      await createMutation.mutateAsync(values);
      message.success(t('common.operate_success'));
      setModalOpen(false);
    } catch (err) {
      console.error('create outbound failed', err);
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

  const columns = [
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
        <Space>
          {record.approval_status === 'pending' && (
            <>
              <Button
                type="link"
                onClick={() => handleApprove(record.id)}
                loading={approveMutation.isPending}
              >
                {t('outbound.approve')}
              </Button>
              <Button
                type="link"
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
            placeholder={t('outbound.outbound_type')}
            allowClear
            style={{ width: 130 }}
            value={typeFilter}
            onChange={(v) => {
              setTypeFilter(v);
              setPage(1);
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
              setPage(1);
            }}
            options={[
              { label: t('outbound.status.pending'), value: 'pending' },
              { label: t('outbound.status.approved'), value: 'approved' },
              { label: t('outbound.status.rejected'), value: 'rejected' },
            ]}
          />
        </Space>
        <Button type="primary" icon={<PlusOutlined />} onClick={openCreateModal}>
          {t('outbound.create_outbound')}
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
        title={t('outbound.create_outbound')}
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
              icon={<StockOutlined />}
              onClick={() => setStockSelectorOpen(true)}
              block
            >
              {t('outbound.from_stock')}
            </Button>
          </div>

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
            <Input type="number" />
          </Form.Item>
          <Form.Item name="customer_id" label={t('outbound.customer_id')}>
            <Input type="number" />
          </Form.Item>
          <Form.Item name="notes" label={t('outbound.notes')}>
            <Input.TextArea rows={3} />
          </Form.Item>
          <Form.Item label={t('outbound.pipes')}>
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
                        <Input placeholder={t('outbound.pipe_id_placeholder')} type="number" style={{ width: 120 }} />
                      </Form.Item>
                      {fields.length > 1 && (
                        <Button size="small" danger onClick={() => remove(name)}>
                          {t('common.delete')}
                        </Button>
                      )}
                    </Space>
                  ))}
                  <Button type="dashed" onClick={() => add({ pipe_type: 'casing' })} block>
                    + {t('outbound.add_pipe')}
                  </Button>
                </>
              )}
            </Form.List>
          </Form.Item>
        </Form>
      </Modal>

      <StockSelector
        open={stockSelectorOpen}
        onCancel={() => setStockSelectorOpen(false)}
        onSelect={handleStockSelect}
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
    </div>
  );
}
