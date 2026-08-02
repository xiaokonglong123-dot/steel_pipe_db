import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button, Card, Form, Input, Modal, Select, Space, Table, Tabs, message } from 'antd';
import { PlusOutlined, PlayCircleOutlined, StepForwardOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { manufacturingApi, type Bom, type WorkOrder } from '../api/manufacturingApi';
import { PageLayout } from '@/shared/components/PageLayout';

export default function ManufacturingPage() {
  const { t } = useTranslation('manufacturing');
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const [creating, setCreating] = useState<'bom' | 'wo' | 'ncr' | null>(null);

  const invalidate = () => {
    ['mfg-boms', 'mfg-wo', 'mfg-ncrs'].forEach((k) => queryClient.invalidateQueries({ queryKey: [k] }));
  };

  const { data: boms } = useQuery({ queryKey: ['mfg-boms'], queryFn: manufacturingApi.listBoms });
  const { data: workOrders } = useQuery({ queryKey: ['mfg-wo'], queryFn: manufacturingApi.listWorkOrders });
  const { data: ncrs } = useQuery({ queryKey: ['mfg-ncrs'], queryFn: manufacturingApi.listNcrs });

  const createBom = useMutation({
    mutationFn: manufacturingApi.createBom,
    onSuccess: () => { message.success(t('saved')); invalidate(); setCreating(null); form.resetFields(); },
  });
  const createWo = useMutation({
    mutationFn: manufacturingApi.createWorkOrder,
    onSuccess: () => { message.success(t('saved')); invalidate(); setCreating(null); form.resetFields(); },
  });
  const createNcr = useMutation({
    mutationFn: manufacturingApi.createNcr,
    onSuccess: () => { message.success(t('saved')); invalidate(); setCreating(null); form.resetFields(); },
  });
  const startWo = useMutation({
    mutationFn: manufacturingApi.startWorkOrder,
    onSuccess: () => { message.success(t('started')); invalidate(); },
  });
  const stepWo = useMutation({
    mutationFn: manufacturingApi.completeStep,
    onSuccess: () => { message.success(t('stepDone')); invalidate(); },
  });

  const handleCreate = async () => {
    const v = await form.validateFields();
    if (creating === 'bom') createBom.mutate({ name: v.name, product_type: v.product_type ?? 'seamless', items: [{ material: v.material ?? 'steel_billet', quantity: 1 }] });
    if (creating === 'wo') createWo.mutate({ bom_id: v.bom_id ? Number(v.bom_id) : undefined, product_type: v.product_type ?? 'seamless', quantity: Number(v.quantity) });
    if (creating === 'ncr') createNcr.mutate({ description: v.description, severity: v.severity });
  };

  const bomColumns = [
    { title: t('name'), dataIndex: 'name', key: 'name' },
    { title: t('productType'), dataIndex: 'product_type', key: 'product_type' },
    { title: t('version'), dataIndex: 'version', key: 'version' },
  ];
  const woColumns = [
    { title: t('woNo'), dataIndex: 'wo_no', key: 'wo_no' },
    { title: t('productType'), dataIndex: 'product_type', key: 'product_type' },
    { title: t('quantity'), dataIndex: 'quantity', key: 'quantity' },
    { title: t('status'), dataIndex: 'status', key: 'status' },
    { title: t('step'), dataIndex: 'current_step', key: 'current_step' },
    { title: t('actions'), key: 'actions', render: (_: unknown, r: WorkOrder) => (
      <Space>
        {r.status === 'pending' && (
          <Button size="small" type="primary" icon={<PlayCircleOutlined />} onClick={() => startWo.mutate(r.id)}>{t('start')}</Button>
        )}
        {r.status === 'in_progress' && (
          <Button size="small" icon={<StepForwardOutlined />} onClick={() => stepWo.mutate(r.id)}>{t('completeStep')}</Button>
        )}
      </Space>
    ) },
  ];
  const ncrColumns = [
    { title: t('ncrNo'), dataIndex: 'ncr_no', key: 'ncr_no' },
    { title: t('description'), dataIndex: 'description', key: 'description' },
    { title: t('severity'), dataIndex: 'severity', key: 'severity' },
    { title: t('status'), dataIndex: 'status', key: 'status' },
  ];

  const tabs = [
    { key: 'boms', label: t('boms'), children: <Table rowKey="id" dataSource={boms ?? []} columns={bomColumns} pagination={false} size="small" /> },
    { key: 'wo', label: t('workOrders'), children: <Table rowKey="id" dataSource={workOrders ?? []} columns={woColumns} pagination={false} size="small" /> },
    { key: 'ncrs', label: t('ncrs'), children: <Table rowKey="id" dataSource={ncrs ?? []} columns={ncrColumns} pagination={false} size="small" /> },
  ];

  return (
    <PageLayout title={t('title')}>
      <Card>
        <Space style={{ marginBottom: 16 }}>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => setCreating('bom')}>{t('newBom')}</Button>
          <Button icon={<PlusOutlined />} onClick={() => setCreating('wo')}>{t('newWo')}</Button>
          <Button danger icon={<PlusOutlined />} onClick={() => setCreating('ncr')}>{t('newNcr')}</Button>
        </Space>
        <Tabs items={tabs} />
      </Card>

      <Modal title={t(`new_${creating ?? 'bom'}`)} open={!!creating} onCancel={() => setCreating(null)} onOk={handleCreate} okText={t('save')} cancelText={t('cancel')}>
        <Form form={form} layout="vertical">
          {creating === 'bom' && (
            <>
              <Form.Item name="name" label={t('name')} rules={[{ required: true }]}><Input /></Form.Item>
              <Form.Item name="product_type" label={t('productType')} initialValue="seamless"><Input /></Form.Item>
              <Form.Item name="material" label={t('material')} initialValue="steel_billet"><Input /></Form.Item>
            </>
          )}
          {creating === 'wo' && (
            <>
              <Form.Item name="bom_id" label={t('bomId')}>
                <Select allowClear options={(boms ?? []).map((b: Bom) => ({ value: b.id, label: b.name }))} />
              </Form.Item>
              <Form.Item name="product_type" label={t('productType')} initialValue="seamless"><Input /></Form.Item>
              <Form.Item name="quantity" label={t('quantity')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
            </>
          )}
          {creating === 'ncr' && (
            <>
              <Form.Item name="description" label={t('description')} rules={[{ required: true }]}><Input.TextArea rows={2} /></Form.Item>
              <Form.Item name="severity" label={t('severity')} initialValue="minor">
                <Select options={['minor', 'major', 'critical'].map((v) => ({ value: v, label: v }))} />
              </Form.Item>
            </>
          )}
        </Form>
      </Modal>
    </PageLayout>
  );
}
