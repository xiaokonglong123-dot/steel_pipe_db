import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button, Card, Descriptions, Form, Input, Modal, Space, Table, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { projectApi, type Project } from '../api/projectApi';
import { PageLayout } from '@/shared/components/PageLayout';

export default function ProjectPage() {
  const { t } = useTranslation('project');
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const [creating, setCreating] = useState<'project' | 'wbs' | null>(null);
  const [selected, setSelected] = useState<number | null>(null);

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ['projects'] });
    queryClient.invalidateQueries({ queryKey: ['wbs'] });
    queryClient.invalidateQueries({ queryKey: ['fin'] });
  };

  const { data: projects } = useQuery({ queryKey: ['projects'], queryFn: projectApi.listProjects });
  const { data: wbs } = useQuery({
    queryKey: ['wbs', selected],
    queryFn: () => (selected ? projectApi.wbsTree(selected) : Promise.resolve([])),
    enabled: !!selected,
  });
  const { data: fin } = useQuery({
    queryKey: ['fin', selected],
    queryFn: () => projectApi.financials(selected!),
    enabled: !!selected,
  });

  const createProject = useMutation({
    mutationFn: projectApi.createProject,
    onSuccess: () => { message.success(t('saved')); invalidate(); setCreating(null); form.resetFields(); },
  });
  const createWbs = useMutation({
    mutationFn: ({ projectId, data }: { projectId: number; data: { code: string; name: string; weight_pct?: number } }) =>
      projectApi.createWbs(projectId, data),
    onSuccess: () => { message.success(t('saved')); invalidate(); setCreating(null); form.resetFields(); },
  });
  const updateStatus = useMutation({
    mutationFn: ({ id, status }: { id: number; status: string }) => projectApi.updateStatus(id, status),
    onSuccess: () => { message.success(t('updated')); invalidate(); },
  });

  const handleCreate = async () => {
    const v = await form.validateFields();
    if (creating === 'project') createProject.mutate({ name: v.name, budget: v.budget ? Number(v.budget) : undefined });
    if (creating === 'wbs' && selected) createWbs.mutate({ projectId: selected, data: { code: v.code, name: v.name, weight_pct: v.weight_pct ? Number(v.weight_pct) : undefined } });
  };

  const projectColumns = [
    { title: t('projectNo'), dataIndex: 'project_no', key: 'project_no' },
    { title: t('name'), dataIndex: 'name', key: 'name' },
    { title: t('budget'), dataIndex: 'budget', key: 'budget' },
    { title: t('status'), dataIndex: 'status', key: 'status' },
    { title: t('actions'), key: 'actions', render: (_: unknown, r: Project) => (
      <Space>
        <Button size="small" type="primary" onClick={() => setSelected(r.id)}>{t('view')}</Button>
        {r.status === 'planning' && (
          <Button size="small" onClick={() => updateStatus.mutate({ id: r.id, status: 'active' })}>{t('activate')}</Button>
        )}
      </Space>
    ) },
  ];
  const wbsColumns = [
    { title: t('code'), dataIndex: 'code', key: 'code' },
    { title: t('name'), dataIndex: 'name', key: 'name' },
    { title: t('parent'), dataIndex: 'parent_id', key: 'parent_id', render: (v: number | null) => v ?? '-' },
    { title: t('progress'), dataIndex: 'progress_pct', key: 'progress_pct' },
  ];

  return (
    <PageLayout title={t('title')}>
      <Card>
        <Space style={{ marginBottom: 16 }}>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => setCreating('project')}>{t('newProject')}</Button>
          <Button icon={<PlusOutlined />} disabled={!selected} onClick={() => setCreating('wbs')}>{t('newWbs')}</Button>
        </Space>
        <Table rowKey="id" dataSource={projects ?? []} columns={projectColumns} pagination={false} size="small" />
      </Card>

      {selected && (
        <Card title={t('projectDetail')} style={{ marginTop: 16 }}>
          {fin && (
            <Descriptions column={3} size="small" style={{ marginBottom: 16 }}>
              <Descriptions.Item label={t('budget')}>{fin.budget}</Descriptions.Item>
              <Descriptions.Item label={t('expense')}>{fin.expense_total}</Descriptions.Item>
              <Descriptions.Item label={t('remaining')}>{fin.remaining}</Descriptions.Item>
            </Descriptions>
          )}
          <Table rowKey="id" dataSource={wbs ?? []} columns={wbsColumns} pagination={false} size="small" />
        </Card>
      )}

      <Modal title={creating === 'project' ? t('newProject') : t('newWbs')} open={!!creating} onCancel={() => setCreating(null)} onOk={handleCreate} okText={t('save')} cancelText={t('cancel')}>
        <Form form={form} layout="vertical">
          {creating === 'project' && (
            <>
              <Form.Item name="name" label={t('name')} rules={[{ required: true }]}><Input /></Form.Item>
              <Form.Item name="budget" label={t('budget')}><Input type="number" /></Form.Item>
            </>
          )}
          {creating === 'wbs' && (
            <>
              <Form.Item name="code" label={t('code')} rules={[{ required: true }]}><Input /></Form.Item>
              <Form.Item name="name" label={t('name')} rules={[{ required: true }]}><Input /></Form.Item>
              <Form.Item name="weight_pct" label={t('weight')}><Input type="number" /></Form.Item>
            </>
          )}
        </Form>
      </Modal>
    </PageLayout>
  );
}
