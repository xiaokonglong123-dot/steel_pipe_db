import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button, Card, Form, Input, Modal, Space, Table, Tabs, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { atpApi } from '../api/atpApi';
import { PageLayout } from '@/shared/components/PageLayout';

export default function InventoryAtpPage() {
  const { t } = useTranslation('inventory');
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const [creating, setCreating] = useState<'reserve' | 'transfer' | 'template' | null>(null);

  const invalidate = () => {
    ['atp-overview', 'transfers'].forEach((k) => queryClient.invalidateQueries({ queryKey: [k] }));
  };

  const { data: overview } = useQuery({ queryKey: ['atp-overview'], queryFn: atpApi.overview });
  const { data: transfers } = useQuery({ queryKey: ['transfers'], queryFn: atpApi.listTransfers });

  const reserve = useMutation({
    mutationFn: atpApi.reserve,
    onSuccess: () => { message.success(t('reserved')); invalidate(); setCreating(null); form.resetFields(); },
  });
  const createTransfer = useMutation({
    mutationFn: atpApi.createTransfer,
    onSuccess: () => { message.success(t('transferred')); invalidate(); setCreating(null); form.resetFields(); },
  });
  const createTemplate = useMutation({
    mutationFn: atpApi.createCountTemplate,
    onSuccess: () => { message.success(t('saved')); setCreating(null); form.resetFields(); },
  });

  const handleCreate = async () => {
    const v = await form.validateFields();
    if (creating === 'reserve') reserve.mutate({ pipe_type: v.pipe_type ?? 'seamless', pipe_number: v.pipe_number, quantity: Number(v.quantity) });
    if (creating === 'transfer') createTransfer.mutate({ from_location_id: Number(v.from), to_location_id: Number(v.to), pipe_number: v.pipe_number, quantity: Number(v.quantity) });
    if (creating === 'template') createTemplate.mutate({ name: v.name, location_ids: String(v.location_ids).split(',').map((s: string) => Number(s.trim())) });
  };

  const atpColumns = [
    { title: t('pipeType'), dataIndex: 'pipe_type', key: 'pipe_type' },
    { title: t('onHand'), dataIndex: 'on_hand', key: 'on_hand' },
    { title: t('reserved'), dataIndex: 'reserved', key: 'reserved' },
    { title: t('available'), dataIndex: 'available', key: 'available' },
  ];
  const transferColumns = [
    { title: t('transferNo'), dataIndex: 'transfer_no', key: 'transfer_no' },
    { title: t('from'), dataIndex: 'from_location_id', key: 'from_location_id' },
    { title: t('to'), dataIndex: 'to_location_id', key: 'to_location_id' },
    { title: t('pipeNumber'), dataIndex: 'pipe_number', key: 'pipe_number', render: (v: string | null) => v ?? '-' },
    { title: t('quantity'), dataIndex: 'quantity', key: 'quantity' },
    { title: t('status'), dataIndex: 'status', key: 'status' },
  ];

  const tabs = [
    { key: 'atp', label: t('atpTitle'), children: (
      <Table rowKey="pipe_type" dataSource={overview ?? []} columns={atpColumns} pagination={false} size="small" />
    ) },
    { key: 'transfers', label: t('transfers'), children: (
      <Table rowKey="id" dataSource={transfers ?? []} columns={transferColumns} pagination={false} size="small" />
    ) },
  ];

  return (
    <PageLayout title={t('atpTitle')}>
      <Card>
        <Space style={{ marginBottom: 16 }}>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => setCreating('reserve')}>{t('reserve')}</Button>
          <Button icon={<PlusOutlined />} onClick={() => setCreating('transfer')}>{t('newTransfer')}</Button>
          <Button icon={<PlusOutlined />} onClick={() => setCreating('template')}>{t('newTemplate')}</Button>
        </Space>
        <Tabs items={tabs} />
      </Card>

      <Modal title={t(`new_${creating ?? 'reserve'}`)} open={!!creating} onCancel={() => setCreating(null)} onOk={handleCreate} okText={t('save')} cancelText={t('cancel')}>
        <Form form={form} layout="vertical">
          {creating === 'reserve' && (
            <>
              <Form.Item name="pipe_type" label={t('pipeType')} initialValue="seamless"><Input /></Form.Item>
              <Form.Item name="pipe_number" label={t('pipeNumber')}><Input /></Form.Item>
              <Form.Item name="quantity" label={t('quantity')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
            </>
          )}
          {creating === 'transfer' && (
            <>
              <Form.Item name="from" label={t('fromLocation')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
              <Form.Item name="to" label={t('toLocation')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
              <Form.Item name="pipe_number" label={t('pipeNumber')} rules={[{ required: true }]}><Input /></Form.Item>
              <Form.Item name="quantity" label={t('quantity')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
            </>
          )}
          {creating === 'template' && (
            <>
              <Form.Item name="name" label={t('name')} rules={[{ required: true }]}><Input /></Form.Item>
              <Form.Item name="location_ids" label={t('locationIds')} rules={[{ required: true }]}><Input placeholder="1,2,3" /></Form.Item>
            </>
          )}
        </Form>
      </Modal>
    </PageLayout>
  );
}
