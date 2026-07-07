// 库位管理页 — 使用 DataTable + PageLayout + usePagination
import { useState } from 'react';
import {
  Button,
  Tag,
  Input,
  Modal,
  Form,
  InputNumber,
  Switch,
  Popconfirm,
  message,
} from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { DataTable } from '@/shared/components/DataTable';
import { usePagination } from '@/shared/hooks/usePagination';
import {
  useLocations,
  useCreateLocation,
  useUpdateLocation,
  useDeleteLocation,
} from '../hooks/useInventory';
import type { Location, CreateLocationData, UpdateLocationData } from '../api/inventoryApi';

type ModalMode = 'create' | 'edit' | null;

export default function LocationListPage() {
  const { t } = useTranslation();
  const { page, pageSize, onPaginationChange } = usePagination();
  const [modalMode, setModalMode] = useState<ModalMode>(null);
  const [selectedLoc, setSelectedLoc] = useState<Location | null>(null);

  const [form] = Form.useForm<CreateLocationData>();
  const [editForm] = Form.useForm<UpdateLocationData>();

  const { data, isLoading } = useLocations({
    page,
    page_size: pageSize,
  });

  const createMutation = useCreateLocation();
  const updateMutation = useUpdateLocation();
  const deleteMutation = useDeleteLocation();

  const openCreateModal = () => {
    form.resetFields();
    setModalMode('create');
  };

  const openEditModal = (loc: Location) => {
    setSelectedLoc(loc);
    editForm.setFieldsValue({
      description: loc.description ?? undefined,
      capacity: loc.capacity ?? undefined,
      is_active: loc.is_active,
    });
    setModalMode('edit');
  };

  const closeModal = () => {
    setModalMode(null);
    setSelectedLoc(null);
  };

  const handleCreate = async () => {
    try {
      const values = await form.validateFields();
      await createMutation.mutateAsync(values);
      message.success(t('common.operate_success'));
      closeModal();
    } catch (err) {
      console.error('create location failed', err);
    }
  };

  const handleEdit = async () => {
    if (!selectedLoc) return;
    try {
      const values = await editForm.validateFields();
      await updateMutation.mutateAsync({ id: selectedLoc.id, data: values });
      message.success(t('common.operate_success'));
      closeModal();
    } catch (err) {
      console.error('create location failed', err);
    }
  };

  const columns = [
    {
      title: t('location.full_code'),
      dataIndex: 'full_code',
      key: 'full_code',
    },
    {
      title: t('location.zone_code'),
      dataIndex: 'zone_code',
      key: 'zone_code',
    },
    {
      title: t('location.shelf_code'),
      dataIndex: 'shelf_code',
      key: 'shelf_code',
    },
    {
      title: t('location.level_code'),
      dataIndex: 'level_code',
      key: 'level_code',
    },
    {
      title: t('location.capacity'),
      dataIndex: 'capacity',
      key: 'capacity',
      render: (v: number | undefined) => v ?? '-',
    },
    {
      title: t('location.used_count'),
      dataIndex: 'used_count',
      key: 'used_count',
    },
    {
      title: t('location.is_active'),
      dataIndex: 'is_active',
      key: 'is_active',
      render: (active: boolean) => (
        <Tag color={active ? 'green' : 'red'}>
          {active ? t('common.active') : t('common.inactive')}
        </Tag>
      ),
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: Location) => (
        <>
          <Button type="link" size="small" onClick={() => openEditModal(record)}>
            {t('common.edit')}
          </Button>
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
  ];

  return (
    <PageLayout
      title={t('location.title')}
      extra={
        <Button type="primary" icon={<PlusOutlined />} onClick={openCreateModal}>
          {t('location.create')}
        </Button>
      }
    >
      <DataTable<Location>
        columns={columns}
        items={data?.items}
        total={data?.total}
        page={page}
        pageSize={pageSize}
        loading={isLoading}
        onPaginationChange={onPaginationChange}
      />

      <Modal
        title={t('location.create')}
        open={modalMode === 'create'}
        onOk={handleCreate}
        onCancel={closeModal}
        confirmLoading={createMutation.isPending}
        destroyOnClose
      >
        <Form form={form} layout="vertical" style={{ marginTop: 16 }}>
          <Form.Item
            name="zone_code"
            label={t('location.zone_code')}
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Input />
          </Form.Item>
          <Form.Item
            name="shelf_code"
            label={t('location.shelf_code')}
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Input />
          </Form.Item>
          <Form.Item
            name="level_code"
            label={t('location.level_code')}
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Input />
          </Form.Item>
          <Form.Item name="description" label={t('location.description')}>
            <Input.TextArea rows={2} />
          </Form.Item>
          <Form.Item name="capacity" label={t('location.capacity')}>
            <InputNumber style={{ width: '100%' }} min={0} />
          </Form.Item>
        </Form>
      </Modal>

      <Modal
        title={t('location.edit')}
        open={modalMode === 'edit'}
        onOk={handleEdit}
        onCancel={closeModal}
        confirmLoading={updateMutation.isPending}
        destroyOnClose
      >
        <Form form={editForm} layout="vertical" style={{ marginTop: 16 }}>
          <Form.Item name="description" label={t('location.description')}>
            <Input.TextArea rows={2} />
          </Form.Item>
          <Form.Item name="capacity" label={t('location.capacity')}>
            <InputNumber style={{ width: '100%' }} min={0} />
          </Form.Item>
          <Form.Item
            name="is_active"
            label={t('location.is_active')}
            valuePropName="checked"
          >
            <Switch />
          </Form.Item>
        </Form>
      </Modal>
    </PageLayout>
  );
}
