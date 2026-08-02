import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Badge, Button, Card, List, Tag } from 'antd';
import { CheckOutlined, BellOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { notificationApi } from '../api/notificationApi';
import { PageLayout } from '@/shared/components/PageLayout';

export default function NotificationsPage() {
  const { t } = useTranslation('notification');
  const queryClient = useQueryClient();

  const { data: notifications } = useQuery({
    queryKey: ['notifications'],
    queryFn: () => notificationApi.list(),
    refetchInterval: 30000,
  });
  const { data: unread } = useQuery({ queryKey: ['notif-unread'], queryFn: notificationApi.unreadCount });

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ['notifications'] });
    queryClient.invalidateQueries({ queryKey: ['notif-unread'] });
  };

  const markRead = useMutation({
    mutationFn: notificationApi.markRead,
    onSuccess: invalidate,
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
