// 库存盘点页 — 盘点任务创建、逐项核对（期望 vs 实际）、差异标记
import { useState } from 'react';
import {
  Table,
  Button,
  Space,
  Tag,
  Modal,
  Form,
  Select,
  Input,
  message,
} from 'antd';
import { PlusOutlined, EyeOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import {
  useInventoryChecks,
  useInventoryCheck,
  useCreateCheck,
  useSubmitCheckItem,
  useLocations,
} from '../hooks/useInventory';
import type {
  InventoryCheckRecord,
  InventoryCheckItem,
  CreateCheckData,
  SubmitCheckItemData,
  Location,
} from '../api/inventoryApi';

const STATUS_COLOR_MAP: Record<string, string> = {
  InProgress: 'blue',
  Completed: 'green',
  Cancelled: 'red',
};

const getFoundStatusOptions = (t: (key: string) => string) => [
  { label: t('inventory_check.found_status.found'), value: 'found' },
  { label: t('inventory_check.found_status.missing'), value: 'missing' },
  { label: t('inventory_check.found_status.damaged'), value: 'damaged' },
  { label: t('inventory_check.found_status.wrong_location'), value: 'wrong_location' },
];

export default function InventoryCheckListPage() {
  const { t } = useTranslation();
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [createOpen, setCreateOpen] = useState(false);
  const [detailOpen, setDetailOpen] = useState(false);
  const [detailCheckId, setDetailCheckId] = useState<number>(0);
  const [createForm] = Form.useForm<CreateCheckData>();
  const [submitForm] = Form.useForm<SubmitCheckItemData>();
  const [submittingItemId, setSubmittingItemId] = useState<number | null>(null);

  const { data, isLoading } = useInventoryChecks({ page, page_size: pageSize });
  const { data: checkDetail, isLoading: loadingDetail } = useInventoryCheck(detailCheckId);
  const { data: locations } = useLocations({ active_only: true, page_size: 1000 });

  const createMutation = useCreateCheck();
  const submitItemMutation = useSubmitCheckItem();

  const openCreateModal = () => {
    createForm.resetFields();
    setCreateOpen(true);
  };

  const handleCreate = async () => {
    try {
      const values = await createForm.validateFields();
      await createMutation.mutateAsync(values);
      message.success(t('common.operate_success'));
      setCreateOpen(false);
    } catch (err) {
      console.error('create check failed', err);
    }
  };

  const openDetailModal = (id: number) => {
    setDetailCheckId(id);
    setDetailOpen(true);
  };

  const handleSubmitItem = async (item: InventoryCheckItem) => {
    setSubmittingItemId(item.id);
    try {
      const values = await submitForm.validateFields();
      await submitItemMutation.mutateAsync({
        checkId: detailCheckId,
        itemId: item.id,
        data: values,
      });
      message.success(t('common.operate_success'));
      submitForm.resetFields();
    } catch (err) {
      console.error('create check failed', err);
    } finally {
      setSubmittingItemId(null);
    }
  };

  const listColumns = [
    {
      title: t('inventory_check.check_no'),
      dataIndex: 'check_no',
      key: 'check_no',
    },
    {
      title: t('inventory_check.location'),
      dataIndex: 'location_id',
      key: 'location_id',
      render: (locId: number | undefined) => {
        if (!locId) return '-';
        const loc = locations?.items?.find((l: Location) => l.id === locId);
        return loc?.full_code ?? `#${locId}`;
      },
    },
    {
      title: t('inventory_check.status'),
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => (
        <Tag color={STATUS_COLOR_MAP[status] ?? 'default'}>
          {t(`inventory_check.status.${status.toLowerCase()}`, status)}
        </Tag>
      ),
    },
    {
      title: t('inventory_check.notes'),
      dataIndex: 'notes',
      key: 'notes',
      render: (notes: string | undefined) => notes || '-',
    },
    {
      title: t('inventory_check.created_at'),
      dataIndex: 'created_at',
      key: 'created_at',
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: InventoryCheckRecord) => (
        <Space>
          <Button
            type="link"
            icon={<EyeOutlined />}
            onClick={() => openDetailModal(record.id)}
          >
            {t('inventory_check.view_items')}
          </Button>
        </Space>
      ),
    },
  ];

  const itemColumns = [
    {
      title: t('inventory_check.pipe_type'),
      dataIndex: 'pipe_type',
      key: 'pipe_type',
      render: (v: string) => <Tag>{t('pipe_type.' + v)}</Tag>,
    },
    {
      title: t('inventory_check.pipe_id'),
      dataIndex: 'pipe_id',
      key: 'pipe_id',
    },
    {
      title: t('inventory_check.expected_status'),
      dataIndex: 'expected_status',
      key: 'expected_status',
      render: (v: string) => <Tag>{t('stock.status.' + v)}</Tag>,
    },
    {
      title: t('inventory_check.found_status'),
      dataIndex: 'found_status',
      key: 'found_status',
      render: (v: string | undefined) => {
        if (!v) return <Tag color="orange">{t('inventory_check.pending')}</Tag>;
        const color = v === 'found' ? 'green' : v === 'missing' ? 'red' : 'orange';
        return <Tag color={color}>{t('inventory_check.found_status.' + v)}</Tag>;
      },
    },
    {
      title: t('inventory_check.is_match'),
      dataIndex: 'is_match',
      key: 'is_match',
      render: (v: boolean | undefined | null) => {
        if (v === true) return <Tag color="green">{t('inventory_check.match')}</Tag>;
        if (v === false) return <Tag color="red">{t('inventory_check.mismatch')}</Tag>;
        return '-';
      },
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: InventoryCheckItem) => {
        if (record.found_status) return null;
        return (
          <Space>
            <Select
              size="small"
              style={{ width: 120 }}
              placeholder={t('inventory_check.found_status')}
              options={getFoundStatusOptions(t)}
              onChange={(val) => submitForm.setFieldsValue({ found_status: val })}
            />
            <Button
              size="small"
              type="primary"
              loading={submittingItemId === record.id}
              onClick={() => handleSubmitItem(record)}
            >
              {t('common.save')}
            </Button>
          </Space>
        );
      },
    },
  ];

  return (
    <div>
      <div
        style={{
          display: 'flex',
          justifyContent: 'flex-end',
          marginBottom: 16,
        }}
      >
        <Button type="primary" icon={<PlusOutlined />} onClick={openCreateModal}>
          {t('inventory_check.create')}
        </Button>
      </div>

      <Table
        columns={listColumns}
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
        title={t('inventory_check.create')}
        open={createOpen}
        onOk={handleCreate}
        onCancel={() => setCreateOpen(false)}
        confirmLoading={createMutation.isPending}
        destroyOnClose
      >
        <Form form={createForm} layout="vertical" style={{ marginTop: 16 }}>
          <Form.Item name="location_id" label={t('inventory_check.location')}>
            <Select
              allowClear
              options={(locations?.items ?? []).map((loc) => ({
                label: loc.full_code,
                value: loc.id,
              }))}
            />
          </Form.Item>
          <Form.Item name="notes" label={t('inventory_check.notes')}>
            <Input.TextArea rows={3} />
          </Form.Item>
        </Form>
      </Modal>

      <Modal
        title={`${t('inventory_check.detail')} — ${checkDetail?.record?.check_no ?? ''}`}
        open={detailOpen}
        onCancel={() => setDetailOpen(false)}
        width={900}
        footer={null}
        destroyOnClose
      >
        <Form form={submitForm} layout="inline" style={{ marginBottom: 16 }}>
          <Form.Item name="notes" label={t('inventory_check.notes')} style={{ flex: 1 }}>
            <Input placeholder={t('inventory_check.note_optional')} />
          </Form.Item>
        </Form>
        <Table
          columns={itemColumns}
          dataSource={checkDetail?.items}
          rowKey="id"
          loading={loadingDetail}
          pagination={false}
          size="small"
        />
      </Modal>
    </div>
  );
}
