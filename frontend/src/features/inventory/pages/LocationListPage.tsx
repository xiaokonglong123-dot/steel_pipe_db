import { useState } from 'react';
import {
  Table,
  Button,
  Space,
  Tag,
  Input,
  Modal,
  Form,
  InputNumber,
  Switch,
  Popconfirm,
  message,
} from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
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
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [searchText, setSearchText] = useState('');
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
      description: loc.description,
      capacity: loc.capacity,
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
    } catch {
      // form validation or API error
    }
  };

  const handleEdit = async () => {
    if (!selectedLoc) return;
    try {
      const values = await editForm.validateFields();
      await updateMutation.mutateAsync({ id: selectedLoc.id, data: values });
      message.success(t('common.operate_success'));
      closeModal();
    } catch {
      // form validation or API error
    }
  };

  const filteredData = data?.items?.filter((loc) =>
    searchText
      ? loc.full_code.toLowerCase().includes(searchText.toLowerCase())
      : true,
  );

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
        <Space>
          <Button type="link" onClick={() => openEditModal(record)}>
            {t('common.edit')}
          </Button>
          <Popconfirm
            title="Confirm delete?"
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
        }}
      >
        <Space>
          <Input
            placeholder={t('common.search')}
            prefix={<SearchOutlined />}
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            style={{ width: 250 }}
          />
        </Space>
        <Button type="primary" icon={<PlusOutlined />} onClick={openCreateModal}>
          {t('location.create')}
        </Button>
      </div>

      <Table
        columns={columns}
        dataSource={filteredData}
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
    </div>
  );
}
