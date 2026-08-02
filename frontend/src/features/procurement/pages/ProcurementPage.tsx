import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button, Card, Form, Input, Modal, Space, Table, Tabs, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { procurementApi, type Requisition } from '../api/procurementApi';
import { PageLayout } from '@/shared/components/PageLayout';

export default function ProcurementPage() {
  const { t } = useTranslation('procurement');
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const [creating, setCreating] = useState<'req' | 'quote' | null>(null);

  const invalidate = () => {
    ['reqs', 'quotes'].forEach((k) => queryClient.invalidateQueries({ queryKey: [k] }));
  };

  const { data: reqs } = useQuery({ queryKey: ['reqs'], queryFn: () => procurementApi.listRequisitions() });
  const { data: quotes } = useQuery({ queryKey: ['quotes'], queryFn: procurementApi.listQuotes });

  const createReq = useMutation({
    mutationFn: procurementApi.createRequisition,
    onSuccess: () => { message.success(t('saved')); invalidate(); setCreating(null); form.resetFields(); },
  });
  const createQuote = useMutation({
    mutationFn: procurementApi.createQuote,
    onSuccess: () => { message.success(t('saved')); invalidate(); setCreating(null); form.resetFields(); },
  });
  const updateReq = useMutation({
    mutationFn: ({ id, status }: { id: number; status: string }) => procurementApi.updateRequisitionStatus(id, status),
    onSuccess: () => { message.success(t('updated')); invalidate(); },
  });

  const handleCreate = async () => {
    const v = await form.validateFields();
    if (creating === 'req') createReq.mutate({ title: v.title, items: [{ pipe_type: v.pipe_type ?? 'seamless', quantity: Number(v.quantity) || 1 }] });
    if (creating === 'quote') createQuote.mutate({ supplier_id: Number(v.supplier_id), title: v.title, total_amount: Number(v.total_amount), items: [] });
  };

  const reqColumns = [
    { title: t('reqNo'), dataIndex: 'req_no', key: 'req_no' },
    { title: t('title'), dataIndex: 'title', key: 'title' },
    { title: t('status'), dataIndex: 'status', key: 'status' },
    { title: t('actions'), key: 'actions', render: (_: unknown, r: Requisition) => (
      r.status === 'draft' ? (
        <Button size="small" type="primary" onClick={() => updateReq.mutate({ id: r.id, status: 'submitted' })}>{t('submit')}</Button>
      ) : r.status === 'submitted' ? (
        <Button size="small" onClick={() => updateReq.mutate({ id: r.id, status: 'approved' })}>{t('approve')}</Button>
      ) : null
    ) },
  ];
  const quoteColumns = [
    { title: t('quoteNo'), dataIndex: 'quote_no', key: 'quote_no' },
    { title: t('title'), dataIndex: 'title', key: 'title', render: (v: string | null) => v ?? '-' },
    { title: t('total'), dataIndex: 'total_amount', key: 'total_amount' },
    { title: t('status'), dataIndex: 'status', key: 'status' },
  ];

  const tabs = [
    { key: 'reqs', label: t('requisitions'), children: (
      <Table rowKey="id" dataSource={reqs ?? []} columns={reqColumns} pagination={false} size="small" />
    ) },
    { key: 'quotes', label: t('quotes'), children: (
      <Table rowKey="id" dataSource={quotes ?? []} columns={quoteColumns} pagination={false} size="small" />
    ) },
  ];

  return (
    <PageLayout title={t('title')}>
      <Card>
        <Space style={{ marginBottom: 16 }}>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => setCreating('req')}>{t('newReq')}</Button>
          <Button icon={<PlusOutlined />} onClick={() => setCreating('quote')}>{t('newQuote')}</Button>
        </Space>
        <Tabs items={tabs} />
      </Card>

      <Modal title={creating === 'req' ? t('newReq') : t('newQuote')} open={!!creating} onCancel={() => setCreating(null)} onOk={handleCreate} okText={t('save')} cancelText={t('cancel')}>
        <Form form={form} layout="vertical">
          {creating === 'req' && (
            <>
              <Form.Item name="title" label={t('title')} rules={[{ required: true }]}><Input /></Form.Item>
              <Form.Item name="pipe_type" label={t('pipeType')} initialValue="seamless"><Input /></Form.Item>
              <Form.Item name="quantity" label={t('quantity')} initialValue={1}><Input type="number" /></Form.Item>
            </>
          )}
          {creating === 'quote' && (
            <>
              <Form.Item name="supplier_id" label={t('supplierId')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
              <Form.Item name="title" label={t('title')}><Input /></Form.Item>
              <Form.Item name="total_amount" label={t('total')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
            </>
          )}
        </Form>
      </Modal>
    </PageLayout>
  );
}
