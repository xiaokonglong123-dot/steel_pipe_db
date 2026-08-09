// 采购订单新增/编辑表单页 — 行项按商品(SKU)选择，提交 item_id
import { useEffect, useState } from 'react';
import {
  Form,
  Input,
  DatePicker,
  InputNumber,
  Button,
  Space,
  message,
  Table,
  Popconfirm,
  Card,
} from 'antd';
import { PlusOutlined, DeleteOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout, ItemPicker } from '@/shared/components';
import type { ItemOption } from '@/shared/components';
import type { CreatePurchaseOrderItem, PurchaseOrderItem } from '../types';
import { usePurchase, useCreatePurchaseOrder, useUpdatePurchaseOrder } from '../hooks/usePurchases';

export default function PurchaseOrderFormPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm();
  const [items, setItems] = useState<CreatePurchaseOrderItem[]>([]);
  const [itemModalOpen, setItemModalOpen] = useState(false);

  const isEdit = !!id;
  const orderId = isEdit ? Number(id) : 0;

  const { data: detail, isLoading: loadingOrder } = usePurchase(orderId);
  const orderItems = detail?.items ?? [];
  const createMutation = useCreatePurchaseOrder();
  const updateMutation = useUpdatePurchaseOrder(orderId);

  useEffect(() => {
    if (isEdit && orderItems.length > 0) {
      setItems(
        orderItems.map((item: PurchaseOrderItem) => ({
          item_id: item.item_id,
          sku: item.sku,
          name: item.name,
          quantity: item.quantity,
          unit_price: item.unit_price ?? 0,
          notes: item.notes,
        })),
      );
    }
  }, [isEdit, orderItems]);

  const addItems = (picked: ItemOption[]) => {
    const additions: CreatePurchaseOrderItem[] = picked.map((it) => ({
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

  const updateItem = (index: number, patch: Partial<CreatePurchaseOrderItem>) => {
    setItems((prev) => prev.map((it, i) => (i === index ? { ...it, ...patch } : it)));
  };

  const removeItem = (index: number) => {
    setItems((prev) => prev.filter((_, i) => i !== index));
  };

  const handleSubmit = async (values: Record<string, unknown>) => {
    if (items.length === 0) {
      message.warning(t('purchases.items_required', '请至少添加一个商品'));
      return;
    }
    const payload = {
      ...values,
      order_date: (values.order_date as dayjs.Dayjs)?.format('YYYY-MM-DD'),
      items: items.map((it) => ({
        item_id: it.item_id,
        quantity: it.quantity,
        unit_price: it.unit_price ?? 0,
        notes: it.notes ?? null,
      })),
    };
    try {
      if (isEdit) {
        await updateMutation.mutateAsync(payload as Parameters<typeof updateMutation.mutateAsync>[0]);
      } else {
        await createMutation.mutateAsync(payload as Parameters<typeof createMutation.mutateAsync>[0]);
      }
      message.success(t('common.operate_success'));
      navigate('/purchases');
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  if (isEdit && loadingOrder) {
    return <div>{t('common.loading')}</div>;
  }

  const itemColumns = [
    {
      title: t('purchases.item', '商品'),
      key: 'item',
      render: (_: unknown, record: CreatePurchaseOrderItem) =>
        record.name ? `${record.sku} — ${record.name}` : record.sku || `#${record.item_id}`,
    },
    {
      title: t('purchases.quantity'),
      dataIndex: 'quantity',
      key: 'quantity',
      width: 120,
      render: (_: unknown, record: CreatePurchaseOrderItem, index: number) => (
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
      title: t('purchases.unit_price'),
      dataIndex: 'unit_price',
      key: 'unit_price',
      width: 140,
      render: (_: unknown, record: CreatePurchaseOrderItem, index: number) => (
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
      title: t('purchases.total_price'),
      key: 'total_price',
      width: 140,
      render: (_: unknown, record: CreatePurchaseOrderItem) =>
        ((record.quantity ?? 0) * (record.unit_price ?? 0)).toLocaleString(),
    },
    {
      title: t('purchases.notes'),
      dataIndex: 'notes',
      key: 'notes',
      width: 160,
      render: (_: unknown, record: CreatePurchaseOrderItem, index: number) => (
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
      title={isEdit ? t('purchases.edit_purchase') : t('purchases.create_purchase')}
      onBack={() => navigate('/purchases')}
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        style={{ maxWidth: 1000 }}
      >
        <Form.Item
          label={t('purchases.supplier_id')}
          name="supplier_id"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <InputNumber min={1} style={{ width: 200 }} />
        </Form.Item>

        <Form.Item
          label={t('purchases.order_date')}
          name="order_date"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <DatePicker style={{ width: 200 }} />
        </Form.Item>

        <Form.Item label={t('purchases.expected_delivery')} name="expected_date">
          <DatePicker style={{ width: 200 }} />
        </Form.Item>

        <Form.Item label={t('purchases.notes')} name="notes">
          <Input.TextArea rows={3} style={{ maxWidth: 600 }} />
        </Form.Item>

        <Card
          title={t('purchases.items')}
          extra={
            <Button type="dashed" icon={<PlusOutlined />} onClick={() => setItemModalOpen(true)}>
              {t('purchases.add_item')}
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

        <Form.Item style={{ marginTop: 24 }}>
          <Space>
            <Button
              type="primary"
              htmlType="submit"
              loading={createMutation.isPending || updateMutation.isPending}
            >
              {t('common.save')}
            </Button>
            <Button onClick={() => navigate('/purchases')}>
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
