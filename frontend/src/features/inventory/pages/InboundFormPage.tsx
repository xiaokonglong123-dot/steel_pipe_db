// 入库单新增/编辑表单页 — 行项按商品(SKU)选择，提交 item_id + quantity
import { useEffect, useState } from 'react';
import {
  Form,
  Input,
  Select,
  InputNumber,
  Button,
  Space,
  message,
  Table,
  Popconfirm,
  Card,
} from 'antd';
import { PlusOutlined, DeleteOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout, ItemPicker } from '@/shared/components';
import type { ItemOption } from '@/shared/components';
import { INBOUND_TYPES } from '@/shared/constants';
import { useCreateInbound, useUpdateInbound, useInboundRecord } from '../hooks/useInventory';
import type { CreateInboundData, InboundItem } from '../api/inventoryApi';

interface RowItem {
  item_id: number;
  /** Display-only (not sent to backend): SKU / name of the picked item. */
  sku?: string;
  name?: string;
  quantity: number;
}

export default function InboundFormPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm();
  const [items, setItems] = useState<RowItem[]>([]);
  const [itemModalOpen, setItemModalOpen] = useState(false);

  const isEdit = !!id;
  const orderId = isEdit ? Number(id) : 0;

  const { data: inboundRecord, isLoading: loadingRecord } = useInboundRecord(orderId);
  const createMutation = useCreateInbound();
  const updateMutation = useUpdateInbound(orderId);

  useEffect(() => {
    if (isEdit && inboundRecord) {
      form.setFieldsValue({
        inbound_type: inboundRecord.record.inbound_type,
        order_id: inboundRecord.record.order_id,
        supplier_id: inboundRecord.record.supplier_id,
        notes: inboundRecord.record.notes,
      });
      setItems(
        inboundRecord.items.map((item: InboundItem) => ({
          item_id: item.item_id,
          quantity: item.quantity,
        })),
      );
    }
  }, [isEdit, inboundRecord, form]);

  const addItems = (picked: ItemOption[]) => {
    const additions: RowItem[] = picked.map((it) => ({
      item_id: it.id,
      sku: it.sku,
      name: it.name,
      quantity: 1,
    }));
    setItems((prev) => [...prev, ...additions]);
    setItemModalOpen(false);
  };

  const updateItem = (index: number, patch: Partial<RowItem>) => {
    setItems((prev) => prev.map((it, i) => (i === index ? { ...it, ...patch } : it)));
  };

  const removeItem = (index: number) => {
    setItems((prev) => prev.filter((_, i) => i !== index));
  };

  const handleSubmit = async (values: Record<string, unknown>) => {
    if (items.length === 0) {
      message.error(t('common.required'));
      return;
    }
    try {
      const payload: CreateInboundData = {
        inbound_type: String(values.inbound_type ?? ''),
        order_id: values.order_id != null ? Number(values.order_id) : undefined,
        supplier_id: values.supplier_id != null ? Number(values.supplier_id) : undefined,
        notes: values.notes != null ? String(values.notes) : undefined,
        items: items.map((it) => ({ item_id: it.item_id, quantity: it.quantity })),
      };
      if (isEdit) {
        await updateMutation.mutateAsync(payload);
      } else {
        await createMutation.mutateAsync(payload);
      }
      message.success(t('common.operate_success'));
      navigate('/inventory/inbound');
    } catch (err) {
      console.error('submit inbound failed', err);
      message.error(t('common.operate_failed'));
    }
  };

  if (isEdit && loadingRecord) {
    return <div>{t('common.loading')}</div>;
  }

  const itemColumns = [
    {
      title: t('inbound.item', '商品'),
      key: 'item',
      render: (_: unknown, record: RowItem) =>
        record.name ? `${record.sku ?? ''} — ${record.name}` : record.sku || `#${record.item_id}`,
    },
    {
      title: t('inbound.quantity', '数量'),
      dataIndex: 'quantity',
      key: 'quantity',
      width: 120,
      render: (_: unknown, record: RowItem, index: number) =>
        isEdit ? (
          <span>{record.quantity}</span>
        ) : (
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
      title: t('common.actions'),
      key: 'actions',
      width: 80,
      render: (_: unknown, __: unknown, index: number) =>
        isEdit ? null : (
          <Popconfirm title={t('common.confirm_delete')} onConfirm={() => removeItem(index)}>
            <Button type="link" danger icon={<DeleteOutlined />} />
          </Popconfirm>
        ),
    },
  ];

  return (
    <PageLayout
      title={isEdit ? t('common.edit') : t('inbound.create_inbound')}
      onBack={() => navigate('/inventory/inbound')}
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        style={{ maxWidth: 1000 }}
      >
        <Form.Item
          label={t('inbound.inbound_type')}
          name="inbound_type"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Select style={{ width: 200 }}>
            {INBOUND_TYPES.map((type) => (
              <Select.Option key={type} value={type}>
                {t(`inbound.type.${type}`)}
              </Select.Option>
            ))}
          </Select>
        </Form.Item>

        <Form.Item label={t('inbound.order_id')} name="order_id">
          <InputNumber min={1} style={{ width: 200 }} />
        </Form.Item>

        <Form.Item label={t('inbound.supplier_id')} name="supplier_id">
          <InputNumber min={1} style={{ width: 200 }} />
        </Form.Item>

        <Form.Item label={t('inbound.notes')} name="notes">
          <Input.TextArea rows={3} style={{ maxWidth: 600 }} />
        </Form.Item>

        <Card
          title={t('inbound.items', '入库商品')}
          extra={
            !isEdit && (
              <Button type="dashed" icon={<PlusOutlined />} onClick={() => setItemModalOpen(true)}>
                {t('inbound.add_item', '添加商品')}
              </Button>
            )
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
              loading={isEdit ? updateMutation.isPending : createMutation.isPending}
            >
              {t('common.save')}
            </Button>
            <Button onClick={() => navigate('/inventory/inbound')}>
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
