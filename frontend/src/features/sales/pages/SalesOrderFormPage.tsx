// 销售订单新增/编辑表单页 — 使用 PageLayout
import { useEffect, useState } from 'react';
import {
  Form, Input, DatePicker, InputNumber, Button, Space, message,
  Card, Table, Popconfirm,
} from 'antd';
import { PlusOutlined, DeleteOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout, ItemPicker } from '@/shared/components';
import type { ItemOption } from '@/shared/components';
import { useSalesOrder, useCreateSalesOrder, useUpdateSalesOrder } from '../hooks/useSales';
import type { CreateSalesOrderData, CreateSalesOrderItemData, SalesOrderItem } from '../types';

export default function SalesOrderFormPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm<CreateSalesOrderData>();
  const [items, setItems] = useState<CreateSalesOrderItemData[]>([]);
  const [itemModalOpen, setItemModalOpen] = useState(false);

  const isEdit = !!id;
  const orderId = isEdit ? Number(id) : 0;

  const { data: detail, isLoading: loadingOrder } = useSalesOrder(orderId);
  const order = detail?.order;
  const orderItems = detail?.items ?? [];
  const createMutation = useCreateSalesOrder();
  const updateMutation = useUpdateSalesOrder(orderId);

  useEffect(() => {
    if (isEdit && order && orderItems.length > 0) {
      form.setFieldsValue({
        customer_id: order.customer_id,
        order_date: order.order_date,
        notes: order.notes ?? undefined,
      });
      setItems(
        orderItems.map((item: SalesOrderItem) => ({
          item_id: item.item_id,
          sku: item.sku,
          name: item.name,
          quantity: item.quantity,
          unit_price: item.unit_price ?? 0,
          notes: item.notes ?? undefined,
        })),
      );
    }
  }, [isEdit, order, form, orderItems]);

  const addItems = (picked: ItemOption[]) => {
    const additions: CreateSalesOrderItemData[] = picked.map((it) => ({
      item_id: it.id,
      sku: it.sku,
      name: it.name,
      quantity: 1,
      unit_price: 0,
      notes: undefined,
    }));
    setItems((prev) => [...prev, ...additions]);
    setItemModalOpen(false);
  };

  const updateItem = (index: number, patch: Partial<CreateSalesOrderItemData>) => {
    setItems((prev) => prev.map((it, i) => (i === index ? { ...it, ...patch } : it)));
  };

  const removeItem = (index: number) => {
    setItems((prev) => prev.filter((_, i) => i !== index));
  };

  const handleSubmit = async (values: CreateSalesOrderData) => {
    if (items.length === 0) {
      message.error(t('sales.please_add_item'));
      return;
    }
    try {
      const payload = {
        ...values,
        items: items.map((it) => ({
          item_id: it.item_id,
          quantity: it.quantity,
          unit_price: it.unit_price ?? 0,
          notes: it.notes ?? null,
        })),
      };
      if (isEdit) {
        await updateMutation.mutateAsync(payload as Parameters<typeof updateMutation.mutateAsync>[0]);
      } else {
        await createMutation.mutateAsync(payload as Parameters<typeof createMutation.mutateAsync>[0]);
      }
      message.success(t('common.operate_success'));
      navigate('/sales');
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  if (isEdit && loadingOrder) {
    return <div>{t('common.loading')}</div>;
  }

  const itemColumns = [
    {
      title: t('sales.item', '商品'),
      key: 'item',
      render: (_: unknown, record: CreateSalesOrderItemData) =>
        record.name ? `${record.sku ?? ''} — ${record.name}` : record.sku || `#${record.item_id}`,
    },
    {
      title: t('sales.quantity'),
      dataIndex: 'quantity',
      key: 'quantity',
      width: 120,
      render: (_: unknown, record: CreateSalesOrderItemData, index: number) => (
        <InputNumber
          min={0}
          step={1}
          value={record.quantity}
          style={{ width: '100%' }}
          onChange={(v) => updateItem(index, { quantity: v ?? 0 })}
        />
      ),
    },
    {
      title: t('sales.unit_price'),
      dataIndex: 'unit_price',
      key: 'unit_price',
      width: 140,
      render: (_: unknown, record: CreateSalesOrderItemData, index: number) => (
        <InputNumber
          min={0}
          step={0.01}
          value={record.unit_price}
          style={{ width: '100%' }}
          onChange={(v) => updateItem(index, { unit_price: v ?? 0 })}
        />
      ),
    },
    {
      title: t('sales.total_price'),
      key: 'total_price',
      width: 140,
      render: (_: unknown, record: CreateSalesOrderItemData) =>
        ((record.quantity ?? 0) * (record.unit_price ?? 0)).toLocaleString(),
    },
    {
      title: t('sales.notes'),
      dataIndex: 'notes',
      key: 'notes',
      width: 160,
      render: (_: unknown, record: CreateSalesOrderItemData, index: number) => (
        <Input
          value={record.notes ?? ''}
          onChange={(e) => updateItem(index, { notes: e.target.value })}
        />
      ),
    },
    {
      title: t('common.actions'),
      key: 'actions',
      width: 80,
      render: (_: unknown, __: unknown, index: number) => (
        <Popconfirm title={t('common.confirm_delete')} onConfirm={() => removeItem(index)}>
          <Button type="link" danger icon={<DeleteOutlined />} />
        </Popconfirm>
      ),
    },
  ];

  return (
    <PageLayout
      title={`${isEdit ? t('common.edit') : t('common.create')} ${t('sales.sales_order')}`}
      onBack={() => navigate('/sales')}
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        style={{ maxWidth: 800 }}
      >
        <Form.Item
          label={t('sales.customer_id')}
          name="customer_id"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <InputNumber style={{ width: '100%' }} min={1} />
        </Form.Item>

        <Form.Item label={t('sales.customer_name')} name="customer_name">
          <Input />
        </Form.Item>

        <Form.Item label={t('sales.order_date')} name="order_date" rules={[{ required: true, message: t('common.required') }]}>
          <DatePicker style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item label={t('sales.expected_delivery')} name="expected_delivery">
          <DatePicker style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item label={t('common.notes')} name="notes">
          <Input.TextArea rows={3} />
        </Form.Item>

        <Card
          title={t('sales.items')}
          extra={
            <Button
              type="dashed"
              icon={<PlusOutlined />}
              onClick={() => setItemModalOpen(true)}
            >
              {t('sales.add_item')}
            </Button>
          }
          style={{ marginBottom: 24 }}
        >
          <Table
            columns={itemColumns}
            dataSource={items}
            rowKey={(_, index) => String(index)}
            pagination={false}
          />
        </Card>

        <Form.Item>
          <Space>
            <Button
              type="primary"
              htmlType="submit"
              loading={createMutation.isPending || updateMutation.isPending}
            >
              {t('common.save')}
            </Button>
            <Button onClick={() => navigate('/sales')}>
              {t('common.cancel')}
            </Button>
          </Space>
        </Form.Item>
      </Form>

      <ItemPicker
        open={itemModalOpen}
        onCancel={() => setItemModalOpen(false)}
        onSelect={addItems}
      />
    </PageLayout>
  );
}
