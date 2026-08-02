import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button, Card, Input, Modal, Space, Table, Tag, message } from 'antd';
import { CheckOutlined, CloseOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { workflowApi, type ApprovalTask } from '../api/workflowApi';
import { PageLayout } from '@/shared/components/PageLayout';

export default function MyTasksPage() {
  const { t } = useTranslation('workflow');
  const queryClient = useQueryClient();
  const [rejectTarget, setRejectTarget] = useState<ApprovalTask | null>(null);
  const [reason, setReason] = useState('');

  const { data: tasks, isLoading } = useQuery({
    queryKey: ['workflow-my-tasks'],
    queryFn: workflowApi.myTasks,
    refetchInterval: 15000,
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['workflow-my-tasks'] });

  const approve = useMutation({
    mutationFn: (nodeId: number) => workflowApi.approveTask(nodeId),
    onSuccess: () => {
      message.success(t('approved'));
      invalidate();
    },
  });

  const reject = useMutation({
    mutationFn: ({ nodeId, reason }: { nodeId: number; reason: string }) =>
      workflowApi.rejectTask(nodeId, reason),
    onSuccess: () => {
      message.success(t('rejected'));
      invalidate();
      setRejectTarget(null);
      setReason('');
    },
  });

  const columns = [
    { title: t('task'), dataIndex: 'node_key', key: 'node_key' },
    { title: t('instanceId'), dataIndex: 'instance_id', key: 'instance_id' },
    { title: t('assigneeType'), dataIndex: 'assignee_type', key: 'assignee_type', render: (v: string) => <Tag>{v}</Tag> },
    { title: t('status'), dataIndex: 'status', key: 'status', render: (v: string) => <Tag color={v === 'pending' ? 'processing' : 'default'}>{v}</Tag> },
    { title: t('actions'), key: 'actions', render: (_: unknown, r: ApprovalTask) => (
      <Space>
        <Button size="small" type="primary" icon={<CheckOutlined />} loading={approve.isPending} onClick={() => approve.mutate(r.id)}>
          {t('approve')}
        </Button>
        <Button size="small" danger icon={<CloseOutlined />} onClick={() => setRejectTarget(r)}>
          {t('reject')}
        </Button>
      </Space>
    ) },
  ];

  return (
    <PageLayout title={t('myTasksTitle')}>
      <Card>
        <Table rowKey="id" loading={isLoading} dataSource={tasks ?? []} columns={columns} pagination={false} />
      </Card>

      <Modal
        title={t('reject')}
        open={!!rejectTarget}
        onCancel={() => setRejectTarget(null)}
        onOk={() => rejectTarget && reject.mutate({ nodeId: rejectTarget.id, reason })}
        okButtonProps={{ danger: true, disabled: !reason.trim() }}
        okText={t('reject')}
        cancelText={t('cancel')}
      >
        <Input.TextArea rows={3} value={reason} onChange={(e) => setReason(e.target.value)} placeholder={t('reasonPlaceholder')} />
      </Modal>
    </PageLayout>
  );
}
