import { useEffect, useState } from 'react';
import {
  Form, Input, DatePicker, InputNumber, Button, Space, message,
  Card, Table, Modal, Popconfirm,
} from 'antd';
import { PlusOutlined, DeleteOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useSalesOrder, useCreateSalesOrder, useUpdateSalesOrder } from '../hooks/useSales';
import type { CreateSalesOrderData, CreateSalesOrderItemData, SalesOrderItem } from '../types';

export default function SalesOrderFormPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm<CreateSalesOrderData>();
  const [items, setItems] = useState<CreateSalesOrderItemData[]>([]);

  const isEdit = !!id;
  const orderId = isEdit ? Number(id) : 0;

  const { data: order, isLoading: loadingOrder } = useSalesOrder(orderId);
  const createMutation = useCreateSalesOrder();
  const updateMutation = useUpdateSalesOrder(orderId);

  useEffect(() => {
    if (isEdit && order) {
      form.setFieldsValue({
        customer_id: order.customer_id,
        customer_name: order.customer_name,
        order_date: order.order_date,
        expected_delivery: order.expected_delivery,
        notes: order.notes,
      });
      setItems(
        order.items.map((item: SalesOrderItem) => ({
          pipe_id: item.pipe_id,
          pipe_number: item.pipe_number,
          pipe_type: item.pipe_type,
          grade: item.grade,
          od: item.od,
          wt: item.wt,
          length: item.length,
          quantity: item.quantity,
          unit_price: item.unit_price,
          total_price: item.total_price,
          notes: item.notes,
        })),
      );
    }
  }, [isEdit, order, form]);

  const addItem = (item: CreateSalesOrderItemData) => {
    setItems((prev) => [...prev, item]);
  };

  const removeItem = (index: number) => {
    setItems((prev) => prev.filter((_, i) => i !== index));
  };

  const handleSubmit = async (values: CreateSalesOrderData) => {
    if (items.length === 0) {
      message.error(t('Please add at least one item'));
      return;
    }
    try {
      const payload = { ...values, items };
      if (isEdit) {
        await updateMutation.mutateAsync(payload);
      } else {
        await createMutation.mutateAsync(payload);
      }
      message.success(t('common.operate_success'));
      navigate('/sales');
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  const itemColumns = [
    { title: t('Pipe Number'), dataIndex: 'pipe_number', key: 'pipe_number' },
    { title: t('Pipe Type'), dataIndex: 'pipe_type', key: 'pipe_type' },
    { title: t('Grade'), dataIndex: 'grade', key: 'grade' },
    { title: 'OD', dataIndex: 'od', key: 'od', render: (v: number | null) => v ?? '-' },
    { title: 'WT', dataIndex: 'wt', key: 'wt', render: (v: number | null) => v ?? '-' },
    { title: t('Quantity'), dataIndex: 'quantity', key: 'quantity' },
    { title: t('Unit Price'), dataIndex: 'unit_price', key: 'unit_price', render: (v: number) => v.toLocaleString() },
    { title: t('Total Price'), dataIndex: 'total_price', key: 'total_price', render: (v: number | null) => v?.toLocaleString() ?? '-' },
    {
      title: t('common.actions'), key: 'actions',
      render: (_: unknown, __: unknown, index: number) => (
        <Popconfirm title="确认删除?" onConfirm={() => removeItem(index)}>
          <Button type="link" danger icon={<DeleteOutlined />} />
        </Popconfirm>
      ),
    },
  ];

  if (isEdit && loadingOrder) {
    return <div>{t('common.loading')}</div>;
  }

  return (
    <div>
      <h2 style={{ marginBottom: 24 }}>
        {isEdit ? t('common.edit') : t('common.create')} {t('Sales Order')}
      </h2>
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        style={{ maxWidth: 800 }}
      >
        <Form.Item
          label={t('Customer ID')}
          name="customer_id"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <InputNumber style={{ width: '100%' }} min={1} />
        </Form.Item>

        <Form.Item label={t('Customer Name')} name="customer_name">
          <Input />
        </Form.Item>

        <Form.Item label={t('Order Date')} name="order_date">
          <DatePicker style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item label={t('Expected Delivery')} name="expected_delivery">
          <DatePicker style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item label={t('Notes')} name="notes">
          <Input.TextArea rows={3} />
        </Form.Item>

        <Card
          title={t('Items')}
          extra={
            <Button
              type="dashed"
              icon={<PlusOutlined />}
              onClick={() => {
                Modal.info({
                  title: t('Select Pipe'),
                  content: (
                    <PipeSelector
                      onSelect={(pipe) => {
                        addItem({
                          pipe_id: pipe.id,
                          pipe_number: pipe.pipe_number,
                          pipe_type: pipe.pipe_type,
                          grade: pipe.grade,
                          od: pipe.od,
                          wt: pipe.wt,
                          length: pipe.length,
                          quantity: 1,
                          unit_price: 0,
                        });
                        Modal.destroyAll();
                      }}
                    />
                  ),
                });
              }}
            >
              {t('Add Item')}
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
    </div>
  );
}

function PipeSelector({ onSelect }: { onSelect: (pipe: { id: number; pipe_number: string; pipe_type?: string; grade?: string; od?: number; wt?: number; length?: number }) => void }) {
  const [pipes, setPipes] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const tp = useTranslation();

  useEffect(() => {
    setLoading(true);
    import('@/api/client').then(({ default: apiClient }) =>
      apiClient.get('/inventory?page_size=50').then((res) => {
        setPipes(res.data.data?.items ?? []);
      }).finally(() => setLoading(false)),
    );
  }, []);

  const columns = [
    { title: tp.t('Pipe Number'), dataIndex: 'pipe_number', key: 'pipe_number' },
    { title: tp.t('Grade'), dataIndex: 'grade', key: 'grade' },
    { title: 'OD', dataIndex: 'od', key: 'od' },
    { title: 'WT', dataIndex: 'wt', key: 'wt' },
    {
      title: tp.t('common.actions'), key: 'actions',
      render: (_: unknown, record: typeof pipes[0]) => (
        <Button type="link" onClick={() => onSelect(record)}>
          {tp.t('Select')}
        </Button>
      ),
    },
  ];

  return (
    <Table
      columns={columns}
      dataSource={pipes}
      rowKey="id"
      loading={loading}
      pagination={{ pageSize: 10 }}
    />
  );
}
