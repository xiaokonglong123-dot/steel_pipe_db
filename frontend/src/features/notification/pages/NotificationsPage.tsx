import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Badge, Button, Card, List, Tag, message } from 'antd';
import { CheckOutlined, BellOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { notificationApi } from '../api/notificationApi';
import { notificationQueryKeys } from '../queryKeys';
import { PageLayout } from '@/shared/components/PageLayout';

export default function NotificationsPage() {
  const { t } = useTranslation('notification');
  const queryClient = useQueryClient();

  const { data: notifications } = useQuery({
    queryKey: notificationQueryKeys.list,
    queryFn: () => notificationApi.list(),
    refetchInterval: 30000,
  });
  const { data: unread } = useQuery({ queryKey: notificationQueryKeys.unreadCount, queryFn: notificationApi.unreadCount });

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: notificationQueryKeys.list });
    queryClient.invalidateQueries({ queryKey: notificationQueryKeys.unreadCount });
  };

  const markRead = useMutation({
    mutationFn: notificationApi.markRead,
    onSuccess: invalidate,
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });

  const typeColor = (type: string) => {
    switch (type) {
      case 'workflow': return 'blue';
      case 'finance': return 'green';
      case 'inventory': return 'orange';
      default: return 'default';
    }
  };

  return (
    <PageLayout title={t('title')}>
      <Card>
        <Badge count={unread ?? 0} style={{ marginBottom: 16 }}>
          <BellOutlined style={{ fontSize: 20 }} />
        </Badge>
        <List
          dataSource={notifications ?? []}
          renderItem={(n) => (
            <List.Item
              actions={!n.is_read ? [
                <Button key="read" size="small" type="primary" icon={<CheckOutlined />} onClick={() => markRead.mutate(n.id)}>
                  {t('markRead')}
                </Button>,
              ] : []}
            >
              <List.Item.Meta
                title={<span style={{ fontWeight: n.is_read ? 'normal' : 'bold' }}>{n.title}</span>}
                description={n.content ?? '-'}
              />
              <Tag color={typeColor(n.notify_type)}>{n.notify_type}</Tag>
            </List.Item>
          )}
        />
      </Card>
    </PageLayout>
  );
}
