import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { Button, Card, Form, Input, Modal, Select, Space, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { portalApi } from '../api/portalApi';
import { PageLayout } from '@/shared/components/PageLayout';

export default function PortalAdminPage() {
  const { t } = useTranslation('portal');
  const [form] = Form.useForm();
  const [creating, setCreating] = useState(false);

  const createAccount = useMutation({
    mutationFn: portalApi.createAccount,
    onSuccess: () => { message.success(t('saved')); setCreating(false); form.resetFields(); },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });

  const handleCreate = async () => {
    const v = await form.validateFields();
    createAccount.mutate({
      party_type: v.party_type,
      party_id: Number(v.party_id),
      username: v.username,
      password: v.password,
    });
  };

  return (
    <PageLayout title={t('title')}>
      <Card>
        <Space style={{ marginBottom: 16 }}>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => setCreating(true)}>{t('newAccount')}</Button>
        </Space>
        <p style={{ color: '#999' }}>{t('hint')}</p>
        <p style={{ color: '#999' }}>
          POST /api/v1/portal-api/login → 供应商/客户 JWT → GET /api/v1/portal-api/purchases | /sales
        </p>
      </Card>

      <Modal title={t('newAccount')} open={creating} onCancel={() => setCreating(false)} onOk={handleCreate} okText={t('save')} cancelText={t('cancel')}>
        <Form form={form} layout="vertical">
          <Form.Item name="party_type" label={t('partyType')} initialValue="supplier">
            <Select options={[{ value: 'supplier', label: t('supplier') }, { value: 'customer', label: t('customer') }]} />
          </Form.Item>
          <Form.Item name="party_id" label={t('partyId')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
          <Form.Item name="username" label={t('username')} rules={[{ required: true }]}><Input /></Form.Item>
          <Form.Item name="password" label={t('password')} rules={[{ required: true, min: 6 }]}><Input.Password /></Form.Item>
        </Form>
      </Modal>
    </PageLayout>
  );
}
