import { useState } from 'react';
import { Card, Form, Input, Button, message } from 'antd';
import { useTranslation } from 'react-i18next';
import { useAuthStore } from '@/stores/authStore';
import { useUpdateProfile, useChangePassword } from '../api';

export default function ProfileSettingsPage() {
  const { t } = useTranslation('profile');
  const { t: tc } = useTranslation();
  const user = useAuthStore((s) => s.user);
  const [profileForm] = Form.useForm();
  const [passwordForm] = Form.useForm();
  const [profileLoading, setProfileLoading] = useState(false);
  const [passwordLoading, setPasswordLoading] = useState(false);

  const updateProfile = useUpdateProfile();
  const changePassword = useChangePassword();

  const handleProfileSubmit = async () => {
    if (!user) return;
    try {
      const values = await profileForm.validateFields();
      setProfileLoading(true);
      await updateProfile.mutateAsync({
        id: user.id,
        display_name: values.display_name,
        email: values.email || undefined,
        phone: values.phone || undefined,
      });
      message.success(t('profile.success'));
    } catch {
      // validation or mutation error handled by Ant Design / interceptor
    } finally {
      setProfileLoading(false);
    }
  };

  const handlePasswordSubmit = async () => {
    try {
      const values = await passwordForm.validateFields();
      if (values.new_password !== values.confirm_password) {
        message.error(t('profile.passwordMismatch'));
        return;
      }
      setPasswordLoading(true);
      await changePassword.mutateAsync({
        current_password: values.current_password,
        new_password: values.new_password,
      });
      message.success(t('profile.passwordChanged'));
      passwordForm.resetFields();
    } catch {
      // validation or mutation error handled by Ant Design / interceptor
    } finally {
      setPasswordLoading(false);
    }
  };

  return (
    <div>
      <Card title={t('profile.profileInfo')} style={{ marginBottom: 24 }}>
        <Form
          form={profileForm}
          layout="vertical"
          initialValues={{
            display_name: user?.display_name ?? '',
            email: user?.email ?? '',
            phone: user?.phone ?? '',
          }}
          style={{ maxWidth: 480 }}
        >
          <Form.Item
            label={t('profile.displayName')}
            name="display_name"
            rules={[{ required: true, message: tc('common.required') }]}
          >
            <Input />
          </Form.Item>
          <Form.Item label={t('profile.email')} name="email">
            <Input type="email" />
          </Form.Item>
          <Form.Item label={t('profile.phone')} name="phone">
            <Input />
          </Form.Item>
          <Form.Item>
            <Button
              type="primary"
              loading={profileLoading || updateProfile.isPending}
              onClick={handleProfileSubmit}
            >
              {t('profile.save')}
            </Button>
          </Form.Item>
        </Form>
      </Card>

      <Card title={t('profile.changePassword')}>
        <Form
          form={passwordForm}
          layout="vertical"
          style={{ maxWidth: 480 }}
        >
          <Form.Item
            label={t('profile.currentPassword')}
            name="current_password"
            rules={[{ required: true, message: tc('common.required') }]}
          >
            <Input.Password />
          </Form.Item>
          <Form.Item
            label={t('profile.newPassword')}
            name="new_password"
            rules={[{ required: true, message: tc('common.required') }]}
          >
            <Input.Password />
          </Form.Item>
          <Form.Item
            label={t('profile.confirmPassword')}
            name="confirm_password"
            rules={[
              { required: true, message: tc('common.required') },
              ({ getFieldValue }) => ({
                validator(_, value) {
                  if (!value || getFieldValue('new_password') === value) {
                    return Promise.resolve();
                  }
                  return Promise.reject(new Error(t('profile.passwordMismatch')));
                },
              }),
            ]}
          >
            <Input.Password />
          </Form.Item>
          <Form.Item>
            <Button
              type="primary"
              loading={passwordLoading || changePassword.isPending}
              onClick={handlePasswordSubmit}
            >
              {t('profile.save')}
            </Button>
          </Form.Item>
        </Form>
      </Card>
    </div>
  );
}
