import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button, Card, Form, Input, Modal, Popconfirm, Space, Table, Tag, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { workflowApi, type WorkflowDefinition } from '../api/workflowApi';
import { workflowQueryKeys } from '../queryKeys';
import { PageLayout } from '@/shared/components/PageLayout';

export default function WorkflowListPage() {
  const { t } = useTranslation('workflow');
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const [creating, setCreating] = useState(false);

  const { data: definitions, isLoading } = useQuery<WorkflowDefinition[]>({
    queryKey: workflowQueryKeys.definitions,
    queryFn: workflowApi.listDefinitions,
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: workflowQueryKeys.definitions });

  const createDef = useMutation({
    mutationFn: workflowApi.createDefinition,
    onSuccess: () => {
      message.success(t('saved'));
      invalidate();
      setCreating(false);
      form.resetFields();
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });

  const deleteDef = useMutation({
    mutationFn: workflowApi.deleteDefinition,
    onSuccess: () => {
      message.success(t('deleted'));
      invalidate();
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });

  const handleSubmit = async () => {
    const values = await form.validateFields();
    const nodes = [
      {
        node_key: 'manager',
        assignee_type: 'role',
        assignee_value: 'admin',
      },
      {
        node_key: 'director',
        assignee_type: 'role',
        assignee_value: 'admin',
        condition: { amount_gt: Number(values.amount_threshold) || 50000 },
      },
    ];
    createDef.mutate({
      name: values.name,
      entity_type: values.entity_type || 'purchase_order',
      description: values.description,
      nodes,
    });
  };

  const columns = [
    { title: t('name'), dataIndex: 'name', key: 'name' },
    { title: t('entityType'), dataIndex: 'entity_type', key: 'entity_type', render: (v: string) => <Tag>{v}</Tag> },
    { title: t('status'), dataIndex: 'is_active', key: 'is_active', render: (v: boolean) => v ? <Tag color="green">{t('active')}</Tag> : <Tag>{t('inactive')}</Tag> },
    { title: t('actions'), key: 'actions', render: (_: unknown, r: WorkflowDefinition) => (
      <Popconfirm title={t('confirmDelete')} onConfirm={() => deleteDef.mutate(r.id)}>
        <Button size="small" danger>{t('delete')}</Button>
      </Popconfirm>
    ) },
  ];

  return (
    <PageLayout title={t('title')}>
      <Card>
        <Space style={{ marginBottom: 16 }}>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => setCreating(true)}>
            {t('create')}
          </Button>
        </Space>
        <Table rowKey="id" loading={isLoading} dataSource={definitions ?? []} columns={columns} pagination={false} />
      </Card>

      <Modal title={t('create')} open={creating} onCancel={() => setCreating(false)} footer={null}>
        <Form form={form} layout="vertical" onFinish={handleSubmit}>
          <Form.Item name="name" label={t('name')} rules={[{ required: true, max: 200 }]}>
            <Input />
          </Form.Item>
          <Form.Item name="entity_type" label={t('entityType')} initialValue="purchase_order">
            <Input />
          </Form.Item>
          <Form.Item name="amount_threshold" label={t('amountThreshold')} initialValue={50000}>
            <Input type="number" />
          </Form.Item>
          <Form.Item name="description" label={t('description')}>
            <Input.TextArea rows={2} />
          </Form.Item>
          <Space>
            <Button type="primary" htmlType="submit" loading={createDef.isPending}>{t('save')}</Button>
            <Button onClick={() => setCreating(false)}>{t('cancel')}</Button>
          </Space>
        </Form>
      </Modal>
    </PageLayout>
  );
}
