import { useState } from 'react';
import {
  Table,
  Button,
  Space,
  Tag,
  Input,
  Modal,
  Form,
  Select,
  Switch,
  message,
} from 'antd';
import { PlusOutlined, SearchOutlined, KeyOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { useUsers, useCreateUser, useUpdateUser, useChangePassword } from '../hooks/useUsers';
import type { UserInfo } from '@/types';
import type { CreateUserData, UpdateUserData, ChangePasswordData } from '../api/userApi';

const getRoleOptions = (t: (key: string) => string) => [
  { label: t('user.role.admin'), value: 'admin' },
  { label: t('user.role.warehouse'), value: 'warehouse' },
  { label: t('user.role.qc'), value: 'qc' },
  { label: t('user.role.sales'), value: 'sales' },
];

const ROLE_COLOR_MAP: Record<string, string> = {
  admin: 'red',
  warehouse: 'blue',
  qc: 'green',
  sales: 'orange',
};

type ModalMode = 'create' | 'edit' | 'changePassword' | null;

export default function UserManagementPage() {
  const { t } = useTranslation();
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [searchText, setSearchText] = useState('');
  const [modalMode, setModalMode] = useState<ModalMode>(null);
  const [selectedUser, setSelectedUser] = useState<UserInfo | null>(null);

  const [createForm] = Form.useForm<CreateUserData>();
  const [editForm] = Form.useForm<UpdateUserData>();
  const [passwordForm] = Form.useForm<ChangePasswordData>();

  const { data, isLoading } = useUsers({
    page,
    page_size: pageSize,
    q: searchText || undefined,
  });

  const createMutation = useCreateUser();
  const updateMutation = useUpdateUser();
  const changePasswordMutation = useChangePassword();

  const openCreateModal = () => {
    createForm.resetFields();
    setModalMode('create');
  };

  const openEditModal = (user: UserInfo) => {
    setSelectedUser(user);
    editForm.setFieldsValue({
      display_name: user.display_name,
      role: user.role,
      email: user.email ?? undefined,
      phone: user.phone ?? undefined,
    });
    setModalMode('edit');
  };

  const openChangePasswordModal = (user: UserInfo) => {
    setSelectedUser(user);
    passwordForm.resetFields();
    setModalMode('changePassword');
  };

  const closeModal = () => {
    setModalMode(null);
    setSelectedUser(null);
  };

  const handleCreate = async () => {
    try {
      const values = await createForm.validateFields();
      await createMutation.mutateAsync(values);
      message.success(t('common.operate_success'));
      closeModal();
    } catch {
      // form validation failed or mutation error
    }
  };

  const handleEdit = async () => {
    if (!selectedUser) return;
    try {
      const values = await editForm.validateFields();
      await updateMutation.mutateAsync({ id: selectedUser.id, data: values });
      message.success(t('common.operate_success'));
      closeModal();
    } catch {
      // form validation failed or mutation error
    }
  };

  const handleChangePassword = async () => {
    if (!selectedUser) return;
    try {
      const values = await passwordForm.validateFields();
      await changePasswordMutation.mutateAsync({
        id: selectedUser.id,
        data: values,
      });
      message.success(t('common.operate_success'));
      closeModal();
    } catch {
      // form validation failed or mutation error
    }
  };

  const columns = [
    {
      title: t('user.username'),
      dataIndex: 'username',
      key: 'username',
    },
    {
      title: t('user.display_name'),
      dataIndex: 'display_name',
      key: 'display_name',
    },
    {
      title: t('user.role'),
      dataIndex: 'role',
      key: 'role',
      render: (role: string) => (
        <Tag color={ROLE_COLOR_MAP[role] || 'default'}>{role}</Tag>
      ),
    },
    {
      title: t('user.email'),
      dataIndex: 'email',
      key: 'email',
      render: (email: string | undefined) => email || '-',
    },
    {
      title: t('user.phone'),
      dataIndex: 'phone',
      key: 'phone',
      render: (phone: string | undefined) => phone || '-',
    },
    {
      title: t('user.is_active'),
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
      render: (_: unknown, record: UserInfo) => (
        <Space>
          <Button type="link" onClick={() => openEditModal(record)}>
            {t('common.edit')}
          </Button>
          <Button
            type="link"
            icon={<KeyOutlined />}
            onClick={() => openChangePasswordModal(record)}
          >
            {t('user.change_password')}
          </Button>
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
          {t('user.create_user')}
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
        title={t('user.create_user')}
        open={modalMode === 'create'}
        onOk={handleCreate}
        onCancel={closeModal}
        confirmLoading={createMutation.isPending}
        destroyOnClose
      >
        <Form form={createForm} layout="vertical" style={{ marginTop: 16 }}>
          <Form.Item
            name="username"
            label={t('user.username')}
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Input />
          </Form.Item>
          <Form.Item
            name="password"
            label={t('user.password')}
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Input.Password />
          </Form.Item>
          <Form.Item
            name="display_name"
            label={t('user.display_name')}
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Input />
          </Form.Item>
          <Form.Item
            name="role"
            label={t('user.role')}
            rules={[{ required: true, message: t('common.required') }]}
            initialValue="warehouse"
          >
            <Select options={getRoleOptions(t)} />
          </Form.Item>
          <Form.Item name="email" label={t('user.email')}>
            <Input />
          </Form.Item>
          <Form.Item name="phone" label={t('user.phone')}>
            <Input />
          </Form.Item>
        </Form>
      </Modal>

      <Modal
        title={t('user.edit_user')}
        open={modalMode === 'edit'}
        onOk={handleEdit}
        onCancel={closeModal}
        confirmLoading={updateMutation.isPending}
        destroyOnClose
      >
        <Form form={editForm} layout="vertical" style={{ marginTop: 16 }}>
          <Form.Item
            name="display_name"
            label={t('user.display_name')}
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Input />
          </Form.Item>
          <Form.Item
            name="role"
            label={t('user.role')}
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Select options={getRoleOptions(t)} />
          </Form.Item>
          <Form.Item name="email" label={t('user.email')}>
            <Input />
          </Form.Item>
          <Form.Item name="phone" label={t('user.phone')}>
            <Input />
          </Form.Item>
          <Form.Item
            name="is_active"
            label={t('user.is_active')}
            valuePropName="checked"
          >
            <Switch />
          </Form.Item>
        </Form>
      </Modal>

      <Modal
        title={t('user.change_password')}
        open={modalMode === 'changePassword'}
        onOk={handleChangePassword}
        onCancel={closeModal}
        confirmLoading={changePasswordMutation.isPending}
        destroyOnClose
      >
        <Form form={passwordForm} layout="vertical" style={{ marginTop: 16 }}>
          <Form.Item
            name="new_password"
            label={t('user.new_password')}
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Input.Password />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}
