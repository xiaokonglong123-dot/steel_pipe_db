// Login page — username/password auth, useLogin hook handles token storage + redirect on success
import { useState } from 'react';
import { Form, Input, Button, Card, Typography, message } from 'antd';
import { UserOutlined, LockOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { useLogin } from '../hooks/useAuth';

const { Title, Text } = Typography;

export default function LoginPage() {
  const { t } = useTranslation();
  const loginMutation = useLogin();
  const [loading, setLoading] = useState(false);

  // Login submit: handled by TanStack Query mutation, shows generic error on failure (no leaky details)
  const onFinish = async (values: { username: string; password: string }) => {
    setLoading(true);
    try {
      await loginMutation.mutateAsync(values);
    } catch {
      message.error(t('common.operate_failed'));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div
      style={{
        height: '100vh',
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        background: '#f0f2f5',
      }}
    >
      <Card style={{ width: 400 }}>
        <div style={{ textAlign: 'center', marginBottom: 24 }}>
          <Title level={3}>{t('app.title')}</Title>
          <Text type="secondary">{t('app.subtitle')}</Text>
        </div>
        <Form
          name="login"
          onFinish={onFinish}
          autoComplete="off"
          size="large"
        >
          <Form.Item
            name="username"
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Input
              prefix={<UserOutlined />}
              placeholder={t('user.username')}
            />
          </Form.Item>
          <Form.Item
            name="password"
            rules={[{ required: true, message: t('common.required') }]}
          >
            <Input.Password
              prefix={<LockOutlined />}
              placeholder={t('user.password')}
            />
          </Form.Item>
          <Form.Item>
            <Button
              type="primary"
              htmlType="submit"
              loading={loading}
              block
            >
              {t('user.login')}
            </Button>
          </Form.Item>
        </Form>
      </Card>
    </div>
  );
}
