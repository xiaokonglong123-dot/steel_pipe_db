import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button, Card, Form, Input, Modal, Space, Table, Tag, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { assetsApi, type FixedAsset } from '../api/assetsApi';
import { PageLayout } from '@/shared/components/PageLayout';

export default function AssetsPage() {
  const { t } = useTranslation('assets');
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const [creating, setCreating] = useState(false);

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['assets'] });

  const { data: assets, isLoading } = useQuery({ queryKey: ['assets'], queryFn: assetsApi.list });

  const createAsset = useMutation({
    mutationFn: assetsApi.create,
    onSuccess: () => { message.success(t('saved')); invalidate(); setCreating(false); form.resetFields(); },
  });
  const depreciate = useMutation({
    mutationFn: ({ id, period }: { id: number; period: string }) => assetsApi.depreciate(id, period),
    onSuccess: () => { message.success(t('depreciated')); invalidate(); },
  });
  const dispose = useMutation({
    mutationFn: assetsApi.dispose,
    onSuccess: () => { message.success(t('disposed')); invalidate(); },
  });

  const handleCreate = async () => {
    const v = await form.validateFields();
    createAsset.mutate({
      name: v.name,
      purchase_date: v.purchase_date,
      purchase_cost: Number(v.purchase_cost),
      useful_life_months: v.useful_life_months ? Number(v.useful_life_months) : 60,
    });
  };

  const columns = [
    { title: t('assetNo'), dataIndex: 'asset_no', key: 'asset_no' },
    { title: t('name'), dataIndex: 'name', key: 'name' },
    { title: t('category'), dataIndex: 'category', key: 'category' },
    { title: t('purchaseCost'), dataIndex: 'purchase_cost', key: 'purchase_cost' },
    { title: t('currentValue'), dataIndex: 'current_value', key: 'current_value' },
    { title: t('status'), dataIndex: 'status', key: 'status', render: (v: string) => (
      <Tag color={v === 'active' ? 'green' : 'red'}>{v}</Tag>
    ) },
    { title: t('actions'), key: 'actions', render: (_: unknown, r: FixedAsset) => (
      <Space>
        {r.status === 'active' && (
          <Button size="small" onClick={() => depreciate.mutate({ id: r.id, period: new Date().toISOString().slice(0, 7) })}>
            {t('depreciate')}
          </Button>
        )}
        {r.status === 'active' && (
          <Button size="small" danger onClick={() => dispose.mutate(r.id)}>{t('dispose')}</Button>
        )}
      </Space>
    ) },
  ];

  return (
    <PageLayout title={t('title')}>
      <Card>
        <Space style={{ marginBottom: 16 }}>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => setCreating(true)}>{t('newAsset')}</Button>
        </Space>
        <Table rowKey="id" loading={isLoading} dataSource={assets ?? []} columns={columns} pagination={false} size="small" />
      </Card>

      <Modal title={t('newAsset')} open={creating} onCancel={() => setCreating(false)} onOk={handleCreate} okText={t('save')} cancelText={t('cancel')}>
        <Form form={form} layout="vertical">
          <Form.Item name="name" label={t('name')} rules={[{ required: true }]}><Input /></Form.Item>
          <Form.Item name="purchase_date" label={t('purchaseDate')} rules={[{ required: true }]}><Input type="date" /></Form.Item>
          <Form.Item name="purchase_cost" label={t('purchaseCost')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
          <Form.Item name="useful_life_months" label={t('lifeMonths')} initialValue={60}><Input type="number" /></Form.Item>
        </Form>
      </Modal>
    </PageLayout>
  );
}
