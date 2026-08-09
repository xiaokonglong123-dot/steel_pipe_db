import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button, Card, Form, Input, Modal, Space, Table, Tabs, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { salesCrmApi, type SalesQuote, type Shipment } from '../api/salesCrmApi';
import { salesCrmQueryKeys } from '../queryKeys';
import { PageLayout } from '@/shared/components/PageLayout';

export default function SalesCrmPage() {
  const { t } = useTranslation('sales');
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const [creating, setCreating] = useState<'quote' | 'shipment' | null>(null);

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: salesCrmQueryKeys.quotes });
    queryClient.invalidateQueries({ queryKey: salesCrmQueryKeys.shipments });
  };

  const { data: quotes } = useQuery({ queryKey: salesCrmQueryKeys.quotes, queryFn: salesCrmApi.listQuotes });
  const { data: shipments } = useQuery({ queryKey: salesCrmQueryKeys.shipments, queryFn: salesCrmApi.listShipments });

  const createQuote = useMutation({
    mutationFn: salesCrmApi.createQuote,
    onSuccess: () => { message.success(t('saved')); invalidate(); setCreating(null); form.resetFields(); },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
  const convertQuote = useMutation({
    mutationFn: salesCrmApi.convertQuote,
    onSuccess: (r) => { message.success(`${t('converted')} #${r.order_id}`); invalidate(); },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
  const createShipment = useMutation({
    mutationFn: salesCrmApi.createShipment,
    onSuccess: () => { message.success(t('saved')); invalidate(); setCreating(null); form.resetFields(); },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
  const ship = useMutation({
    mutationFn: ({ id, status }: { id: number; status: string }) => salesCrmApi.updateShipmentStatus(id, status),
    onSuccess: () => { message.success(t('updated')); invalidate(); },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });

  const handleCreate = async () => {
    const v = await form.validateFields();
    if (creating === 'quote') createQuote.mutate({ customer_id: Number(v.customer_id), total_amount: Number(v.total_amount), items: [] });
    if (creating === 'shipment') createShipment.mutate({ sales_order_id: Number(v.sales_order_id), items: [{ quantity: 1 }] });
  };

  const quoteColumns = [
    { title: t('quoteNo'), dataIndex: 'quote_no', key: 'quote_no' },
    { title: t('customerId'), dataIndex: 'customer_id', key: 'customer_id' },
    { title: t('total'), dataIndex: 'total_amount', key: 'total_amount' },
    { title: t('status'), dataIndex: 'status', key: 'status' },
    { title: t('actions'), key: 'actions', render: (_: unknown, r: SalesQuote) => (
      r.status === 'confirmed' ? (
        <Button size="small" type="primary" onClick={() => convertQuote.mutate(r.id)}>{t('convert')}</Button>
      ) : r.status === 'draft' ? (
        <Button size="small" onClick={() => salesCrmApi.updateQuoteStatus(r.id, 'confirmed').then(() => invalidate())}>{t('confirm')}</Button>
      ) : null
    ) },
  ];
  const shipmentColumns = [
    { title: t('shipmentNo'), dataIndex: 'shipment_no', key: 'shipment_no' },
    { title: t('orderId'), dataIndex: 'sales_order_id', key: 'sales_order_id' },
    { title: t('carrier'), dataIndex: 'carrier', key: 'carrier', render: (v: string | null) => v ?? '-' },
    { title: t('status'), dataIndex: 'status', key: 'status' },
    { title: t('actions'), key: 'actions', render: (_: unknown, r: Shipment) => (
      r.status === 'pending' ? (
        <Button size="small" type="primary" onClick={() => ship.mutate({ id: r.id, status: 'shipped' })}>{t('ship')}</Button>
      ) : r.status === 'shipped' ? (
        <Button size="small" onClick={() => ship.mutate({ id: r.id, status: 'delivered' })}>{t('deliver')}</Button>
      ) : null
    ) },
  ];

  const tabs = [
    { key: 'quotes', label: t('quotes'), children: (
      <Table rowKey="id" dataSource={quotes ?? []} columns={quoteColumns} pagination={false} size="small" />
    ) },
    { key: 'shipments', label: t('shipments'), children: (
      <Table rowKey="id" dataSource={shipments ?? []} columns={shipmentColumns} pagination={false} size="small" />
    ) },
  ];

  return (
    <PageLayout title={t('crmTitle')}>
      <Card>
        <Space style={{ marginBottom: 16 }}>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => setCreating('quote')}>{t('newQuote')}</Button>
          <Button icon={<PlusOutlined />} onClick={() => setCreating('shipment')}>{t('newShipment')}</Button>
        </Space>
        <Tabs items={tabs} />
      </Card>

      <Modal title={creating === 'quote' ? t('newQuote') : t('newShipment')} open={!!creating} onCancel={() => setCreating(null)} onOk={handleCreate} okText={t('save')} cancelText={t('cancel')}>
        <Form form={form} layout="vertical">
          {creating === 'quote' && (
            <>
              <Form.Item name="customer_id" label={t('customerId')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
              <Form.Item name="total_amount" label={t('total')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
            </>
          )}
          {creating === 'shipment' && (
            <Form.Item name="sales_order_id" label={t('orderId')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
          )}
        </Form>
      </Modal>
    </PageLayout>
  );
}
