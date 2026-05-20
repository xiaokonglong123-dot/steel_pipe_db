import React, { useState } from 'react';
import {
  Table, Button, Modal, Form, Input, Select, message, Space, Tag, Switch, Popconfirm, Card, Row, Col,
} from 'antd';
import { PlusOutlined, EditOutlined, DeleteOutlined, SearchOutlined } from '@ant-design/icons';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { authApi, type UserInfo } from '../../../api/auth';

const roleOptions = [
  { value: 'admin', labelKey: 'auth.admin' },
  { value: 'warehouse', labelKey: 'auth.warehouse' },
  { value: 'qc', labelKey: 'auth.qc' },
  { value: 'sales', labelKey: 'auth.sales' },
];

const roleColors: Record<string, string> = {
  admin: 'red',
  warehouse: 'blue',
  qc: 'green',
  sales: 'orange',
};

export default function UserListPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [modalOpen, setModalOpen] = useState(false);
  const [editingUser, setEditingUser] = useState<UserInfo | null>(null);
  const [searchText, setSearchText] = useState('');
  const [roleFilter, setRoleFilter] = useState('');
  const [querySearch, setQuerySearch] = useState('');
  const [queryRole, setQueryRole] = useState('');
  const [form] = Form.useForm();

  const handleSearch = () => {
    setQuerySearch(searchText);
    setQueryRole(roleFilter);
  };

  const handleReset = () => {
    setSearchText('');
    setRoleFilter('');
    setQuerySearch('');
    setQueryRole('');
  };

  const { data, isLoading } = useQuery({
    queryKey: ['users', querySearch, queryRole],
    queryFn: async () => {
      const params: { search?: string; role?: string } = {};
      if (querySearch) params.search = querySearch;
      if (queryRole) params.role = queryRole;
      const res = await authApi.listUsers(params);
      return res.data.data;
    },
  });

  const createMutation = useMutation({
    mutationFn: (values: any) => authApi.createUser(values),
    onSuccess: () => {
      message.success(t('common.create') + ' ' + t('common.success'));
      queryClient.invalidateQueries({ queryKey: ['users'], exact: false });
      setModalOpen(false);
      form.resetFields();
    },
    onError: () => message.error(t('common.create') + ' ' + t('common.failed')),
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, values }: { id: string; values: any }) => authApi.updateUser(id, values),
    onSuccess: () => {
      message.success(t('common.edit') + ' ' + t('common.success'));
      queryClient.invalidateQueries({ queryKey: ['users'], exact: false });
      setModalOpen(false);
      setEditingUser(null);
      form.resetFields();
    },
    onError: () => message.error(t('common.edit') + ' ' + t('common.failed')),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => authApi.deleteUser(id),
    onSuccess: () => {
      message.success(t('common.delete') + ' ' + t('common.success'));
      queryClient.invalidateQueries({ queryKey: ['users'], exact: false });
    },
    onError: () => message.error(t('common.delete') + ' ' + t('common.failed')),
  });

  const handleCreate = () => {
    setEditingUser(null);
    form.resetFields();
    setModalOpen(true);
  };

  const handleEdit = (record: UserInfo) => {
    setEditingUser(record);
    form.setFieldsValue({
      username: record.username,
      display_name: record.display_name,
      role: record.role,
      email: record.email,
      phone: record.phone,
      is_active: record.is_active,
    });
    setModalOpen(true);
  };

  const handleSubmit = async () => {
    const values = await form.validateFields();
    if (editingUser) {
      const payload: any = { ...values };
      if (!payload.password) delete payload.password;
      updateMutation.mutate({ id: editingUser.id, values: payload });
    } else {
      createMutation.mutate(values);
    }
  };

  const columns = [
    {
      title: t('auth.username'),
      dataIndex: 'username',
      key: 'username',
      width: 150,
    },
    {
      title: t('auth.displayName'),
      dataIndex: 'display_name',
      key: 'display_name',
      width: 180,
    },
    {
      title: t('auth.role'),
      dataIndex: 'role',
      key: 'role',
      width: 120,
      render: (role: string) => (
        <Tag color={roleColors[role] || 'default'}>{t(`auth.${role}`)}</Tag>
      ),
    },
    {
      title: t('auth.email'),
      dataIndex: 'email',
      key: 'email',
      width: 200,
      render: (v: string | null) => v || '-',
    },
    {
      title: t('auth.phone'),
      dataIndex: 'phone',
      key: 'phone',
      width: 150,
      render: (v: string | null) => v || '-',
    },
    {
      title: t('common.status'),
      key: 'is_active',
      width: 100,
      render: (_: any, record: UserInfo) => (
        record.is_active ? (
          <Tag color="green">{t('common.enabled')}</Tag>
        ) : (
          <Tag color="default">{t('common.disabled')}</Tag>
        )
      ),
    },
    {
      title: t('common.action'),
      key: 'action',
      width: 150,
      render: (_: any, record: UserInfo) => (
        <Space>
          <Button type="link" size="small" icon={<EditOutlined />} onClick={() => handleEdit(record)}>
            {t('common.edit')}
          </Button>
          <Popconfirm
            title={t('common.confirm')}
            description={t('validation.confirmDelete')}
            onConfirm={() => deleteMutation.mutate(record.id)}
          >
            <Button type="link" size="small" danger icon={<DeleteOutlined />}>
              {t('common.delete')}
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <Card title={t('nav.system')} extra={<Button type="primary" icon={<PlusOutlined />} onClick={handleCreate}>{t('common.create')}</Button>}>
      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col>
          <Input
            placeholder="搜索用户名/显示名称"
            prefix={<SearchOutlined />}
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            onPressEnter={handleSearch}
            style={{ width: 240 }}
          />
        </Col>
        <Col>
          <Select value={roleFilter} onChange={setRoleFilter} style={{ width: 140 }}>
            <Select.Option value="">全部角色</Select.Option>
            <Select.Option value="admin">管理员</Select.Option>
            <Select.Option value="warehouse">仓管员</Select.Option>
            <Select.Option value="qc">质检员</Select.Option>
            <Select.Option value="sales">销售员</Select.Option>
          </Select>
        </Col>
        <Col>
          <Space>
            <Button type="primary" onClick={handleSearch}>搜索</Button>
            <Button onClick={handleReset}>重置</Button>
          </Space>
        </Col>
      </Row>
      <Table
        dataSource={data}
        columns={columns}
        rowKey="id"
        loading={isLoading}
        pagination={{ pageSize: 10, showTotal: (total: number) => `${total} ${t('common.total') || 'items'}` }}
      />

      <Modal
        title={editingUser ? t('common.edit') + ' ' + t('auth.user') : t('common.create') + ' ' + t('auth.user')}
        open={modalOpen}
        onOk={handleSubmit}
        onCancel={() => { setModalOpen(false); setEditingUser(null); form.resetFields(); }}
        confirmLoading={createMutation.isPending || updateMutation.isPending}
        width={520}
      >
        <Form form={form} layout="vertical">
          <Form.Item name="username" label={t('auth.username')} rules={editingUser ? undefined : [{ required: true, message: t('validation.required').replace('{{field}}', t('auth.username')) }]}>
            <Input disabled={!!editingUser} />
          </Form.Item>
          <Form.Item name="display_name" label={t('auth.displayName')} rules={[{ required: true, message: t('validation.required').replace('{{field}}', t('auth.displayName')) }]}>
            <Input />
          </Form.Item>
          <Form.Item name="password" label={editingUser ? t('auth.newPassword') : t('auth.password')}
            rules={editingUser ? undefined : [{ required: true, message: t('validation.required').replace('{{field}}', t('auth.password')) }]}>
            <Input.Password placeholder={editingUser ? t('auth.passwordPlaceholder') : ''} />
          </Form.Item>
          <Form.Item name="role" label={t('auth.role')} initialValue="warehouse" rules={[{ required: true }]}>
            <Select>
              {roleOptions.map((opt) => (
                <Select.Option key={opt.value} value={opt.value}>{t(opt.labelKey)}</Select.Option>
              ))}
            </Select>
          </Form.Item>
          <Form.Item name="email" label={t('auth.email')}>
            <Input />
          </Form.Item>
          <Form.Item name="phone" label={t('auth.phone')}>
            <Input />
          </Form.Item>
          {editingUser && (
            <Form.Item name="is_active" label={t('common.status')} valuePropName="checked">
              <Switch checkedChildren={t('common.enabled')} unCheckedChildren={t('common.disabled')} />
            </Form.Item>
          )}
        </Form>
      </Modal>
    </Card>
  );
}
