import { useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button, Card, Form, Input, Modal, Popconfirm, Select, Space, Table, Tag, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { roleApi, type Permission, type Role } from '../api/roleApi';
import { roleQueryKeys } from '../queryKeys';
import { PageLayout } from '@/shared/components/PageLayout';

const PERMISSION_MODULES = ['pipe', 'inventory', 'quality', 'purchase', 'sales', 'finance', 'hr', 'manufacturing', 'workflow', 'system'];

export default function RoleManagementPage() {
  const { t } = useTranslation('auth');
  const queryClient = useQueryClient();
  const [editing, setEditing] = useState<Role | null>(null);
  const [permTarget, setPermTarget] = useState<Role | null>(null);
  const [form] = Form.useForm();

  const { data: roles, isLoading } = useQuery({
    queryKey: roleQueryKeys.roles,
    queryFn: roleApi.listRoles,
  });
  const { data: permissions } = useQuery({
    queryKey: roleQueryKeys.permissions,
    queryFn: roleApi.listPermissions,
  });
  const { data: rolePermissions } = useQuery({
    queryKey: roleQueryKeys.rolePermissions(permTarget?.id),
    queryFn: () => roleApi.getRolePermissions(permTarget!.id),
    enabled: !!permTarget,
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: roleQueryKeys.roles });

  const createRole = useMutation({
    mutationFn: roleApi.createRole,
    onSuccess: () => {
      message.success(t('role.saved'));
      invalidate();
      setEditing(null);
      form.resetFields();
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });

  const updateRole = useMutation({
    mutationFn: (data: { id: number; name?: string; description?: string }) =>
      roleApi.updateRole(data.id, data),
    onSuccess: () => {
      message.success(t('role.saved'));
      invalidate();
      setEditing(null);
      form.resetFields();
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });

  const deleteRole = useMutation({
    mutationFn: roleApi.deleteRole,
    onSuccess: () => {
      message.success(t('common.operate_success'));
      invalidate();
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });

  const savePermissions = useMutation({
    mutationFn: (data: { id: number; permissions: string[] }) =>
      roleApi.setRolePermissions(data.id, data.permissions),
    onSuccess: () => {
      message.success(t('role.permissionsSaved'));
      setPermTarget(null);
      invalidate();
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });

  const permissionsByModule = useMemo(() => {
    const map = new Map<string, Permission[]>();
    for (const p of permissions ?? []) {
      const mod = p.key.split('.')[0] ?? 'other';
      const list = map.get(mod) ?? [];
      list.push(p);
      map.set(mod, list);
    }
    return map;
  }, [permissions]);

  const handleSubmit = async () => {
    const values = await form.validateFields();
    if (editing) {
      updateRole.mutate({ id: editing.id, name: values.name, description: values.description });
    } else {
      createRole.mutate({ name: values.name, description: values.description });
    }
  };

  const columns = [
    { title: t('role.name'), dataIndex: 'name', key: 'name', render: (v: string, r: Role) => (
      <Space>
        {v}
        {r.is_system && <Tag color="blue">{t('role.system')}</Tag>}
      </Space>
    ) },
    { title: t('role.description'), dataIndex: 'description', key: 'description', render: (v: string | null) => v ?? '-' },
    { title: t('common.actions'), key: 'actions', render: (_: unknown, r: Role) => (
      <Space>
        <Button size="small" disabled={r.is_system} onClick={() => { setEditing(r); form.setFieldsValue({ name: r.name, description: r.description }); }}>
          {t('common.edit')}
        </Button>
        <Button size="small" onClick={() => setPermTarget(r)}>
          {t('role.permissions')}
        </Button>
        <Popconfirm title={t('common.confirm_delete')} onConfirm={() => deleteRole.mutate(r.id)} disabled={r.is_system}>
          <Button size="small" danger disabled={r.is_system}>{t('common.delete')}</Button>
        </Popconfirm>
      </Space>
    ) },
  ];

  return (
    <PageLayout title={t('role.title')}>
      <Card>
        <Space style={{ marginBottom: 16 }}>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => { setEditing(null); form.resetFields(); }}>
            {t('role.create')}
          </Button>
        </Space>
        <Table rowKey="id" loading={isLoading} dataSource={roles ?? []} columns={columns} pagination={false} />
      </Card>

      <Modal
        title={editing ? t('role.edit') : t('role.create')}
        open={!!editing || !!permTarget}
        onCancel={() => { setEditing(null); setPermTarget(null); }}
        footer={null}
      >
        {!permTarget && (
          <Form form={form} layout="vertical" onFinish={handleSubmit}>
            <Form.Item name="name" label={t('role.name')} rules={[{ required: true, max: 100 }]}>
              <Input />
            </Form.Item>
            <Form.Item name="description" label={t('role.description')}>
              <Input.TextArea rows={2} />
            </Form.Item>
            <Space>
              <Button type="primary" htmlType="submit" loading={createRole.isPending || updateRole.isPending}>
                {t('common.save')}
              </Button>
              <Button onClick={() => { setEditing(null); setPermTarget(null); }}>{t('common.cancel')}</Button>
            </Space>
          </Form>
        )}
        {permTarget && (
          <Space direction="vertical" style={{ width: '100%' }}>
            {PERMISSION_MODULES.map((mod) => {
              const list = permissionsByModule.get(mod) ?? [];
              if (list.length === 0) return null;
              return (
                <div key={mod}>
                  <Tag color="geekblue">{mod}</Tag>
                  <Select
                    mode="multiple"
                    style={{ width: '100%' }}
                    placeholder={t('role.selectPermissions')}
                    value={(rolePermissions ?? []).filter((k) => k.startsWith(`${mod}.`))}
                    options={list.map((p) => ({ label: p.description ?? p.key, value: p.key }))}
                    onChange={(keys) => {
                      const others = (rolePermissions ?? []).filter((k) => !k.startsWith(`${mod}.`));
                      savePermissions.mutate({ id: permTarget.id, permissions: [...others, ...keys] });
                    }}
                  />
                </div>
              );
            })}
          </Space>
        )}
      </Modal>
    </PageLayout>
  );
}
