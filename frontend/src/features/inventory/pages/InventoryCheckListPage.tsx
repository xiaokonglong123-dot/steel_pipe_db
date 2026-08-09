// 库存盘点页 — 使用 DataTable + PageLayout + usePagination
import { useState } from 'react';
import {
  Button,
  Tag,
  Modal,
  Form,
  Select,
  Input,
  message,
} from 'antd';
import { PlusOutlined, EyeOutlined, CheckCircleOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { DataTable } from '@/shared/components/DataTable';
import { usePagination } from '@/shared/hooks/usePagination';
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
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { inventoryQueryKeys } from '../queryKeys';
import { apiClient } from '@/api/client';

const STATUS_COLOR_MAP: Record<string, string> = {
  in_progress: 'blue',
  completed: 'green',
  cancelled: 'red',
};

const getFoundStatusOptions = (t: (key: string) => string) => [
  { label: t('inventory_check.found_status.found'), value: 'found' },
  { label: t('inventory_check.found_status.missing'), value: 'missing' },
  { label: t('inventory_check.found_status.damaged'), value: 'damaged' },
  { label: t('inventory_check.found_status.wrong_location'), value: 'wrong_location' },
];

export default function InventoryCheckListPage() {
  const { t } = useTranslation();
  const { page, pageSize, onPaginationChange } = usePagination();
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
  const qc = useQueryClient();

  const completeMutation = useMutation({
    mutationFn: (checkId: number) =>
      apiClient.post<{ success: boolean; data: { record: InventoryCheckRecord } }>(
        `/inventory/checks/${checkId}/complete`,
      ),
    onSuccess: () => {
      message.success(t('common.operate_success'));
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.checks.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });

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
        <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
          <Button
            type="link"
            size="small"
            icon={<EyeOutlined />}
            onClick={() => openDetailModal(record.id)}
          >
            {t('inventory_check.view_items')}
          </Button>
          {record.status === 'in_progress' && (
            <Button
              type="link"
              size="small"
              icon={<CheckCircleOutlined />}
              loading={completeMutation.isPending && completeMutation.variables === record.id}
              onClick={() => {
                Modal.confirm({
                  title: t('inventory_check.complete_confirm_title'),
                  content: t('inventory_check.complete_confirm_content'),
                  onOk: () => completeMutation.mutate(record.id),
                });
              }}
            >
              {t('inventory_check.complete')}
            </Button>
          )}
        </div>
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
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
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
          </div>
        );
      },
    },
  ];

  return (
    <PageLayout
      title={t('inventory_check.title')}
      extra={
        <Button type="primary" icon={<PlusOutlined />} onClick={openCreateModal}>
          {t('inventory_check.create')}
        </Button>
      }
    >
      <DataTable<InventoryCheckRecord>
        columns={listColumns}
        items={data?.items}
        total={data?.total}
        page={page}
        pageSize={pageSize}
        loading={isLoading}
        onPaginationChange={onPaginationChange}
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
              options={(locations?.items ?? []).map((loc: Location) => ({
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
        <DataTable<InventoryCheckItem>
          columns={itemColumns}
          items={checkDetail?.items}
          page={1}
          pageSize={checkDetail?.items?.length ?? 100}
          onPaginationChange={() => {}}
          loading={loadingDetail}
        />
      </Modal>
    </PageLayout>
  );
}
