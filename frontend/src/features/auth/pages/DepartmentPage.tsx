import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button, Card, Form, Input, Modal, Popconfirm, Space, Table, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { roleApi, type Department } from '../api/roleApi';
import { departmentQueryKeys } from '../queryKeys';
import { PageLayout } from '@/shared/components/PageLayout';

export default function DepartmentPage() {
  const { t } = useTranslation('auth');
  const queryClient = useQueryClient();
  const [editing, setEditing] = useState<Department | null>(null);
  const [form] = Form.useForm();

  const { data: departments, isLoading } = useQuery({
    queryKey: departmentQueryKeys.all,
    queryFn: roleApi.listDepartments,
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: departmentQueryKeys.all });

  const createDept = useMutation({
    mutationFn: roleApi.createDepartment,
    onSuccess: () => {
      message.success(t('dept.saved'));
      invalidate();
      setEditing(null);
      form.resetFields();
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });

  const deleteDept = useMutation({
    mutationFn: roleApi.deleteDepartment,
    onSuccess: () => {
      message.success(t('common.operate_success'));
      invalidate();
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });

  const handleSubmit = async () => {
    const values = await form.validateFields();
    createDept.mutate({ name: values.name });
  };

  const columns = [
    { title: t('dept.name'), dataIndex: 'name', key: 'name' },
    { title: t('dept.path'), dataIndex: 'path', key: 'path', render: (v: string) => <code>{v}</code> },
    { title: t('common.actions'), key: 'actions', render: (_: unknown, r: Department) => (
      <Space>
        <Button size="small" onClick={() => { setEditing(r); form.setFieldsValue({ name: r.name }); }}>
          {t('common.edit')}
        </Button>
        <Popconfirm title={t('common.confirm_delete')} onConfirm={() => deleteDept.mutate(r.id)}>
          <Button size="small" danger>{t('common.delete')}</Button>
        </Popconfirm>
      </Space>
    ) },
  ];

  return (
    <PageLayout title={t('dept.title')}>
      <Card>
        <Space style={{ marginBottom: 16 }}>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => { setEditing(null); form.resetFields(); }}>
            {t('dept.create')}
          </Button>
        </Space>
        <Table rowKey="id" loading={isLoading} dataSource={departments ?? []} columns={columns} pagination={false} />
      </Card>

      <Modal
        title={editing ? t('dept.edit') : t('dept.create')}
        open={!!editing}
        onCancel={() => { setEditing(null); form.resetFields(); }}
        footer={null}
      >
        <Form form={form} layout="vertical" onFinish={handleSubmit}>
          <Form.Item name="name" label={t('dept.name')} rules={[{ required: true, max: 100 }]}>
            <Input />
          </Form.Item>
          <Space>
            <Button type="primary" htmlType="submit" loading={createDept.isPending}>
              {t('common.save')}
            </Button>
            <Button onClick={() => { setEditing(null); form.resetFields(); }}>{t('common.cancel')}</Button>
          </Space>
        </Form>
      </Modal>
    </PageLayout>
  );
}
