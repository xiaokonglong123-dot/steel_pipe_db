import { useEffect } from 'react';
import {
  Form,
  Input,
  Select,
  DatePicker,
  InputNumber,
  Button,
  Space,
  message,
  Table,
  Popconfirm,
} from 'antd';
import { PlusOutlined, DeleteOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { usePurchase, useCreatePurchaseOrder, useUpdatePurchaseOrder } from '../hooks/usePurchases';

const PIPE_TYPES = ['seamless', 'screen'];
const API_5CT_GRADES = ['H40', 'J55', 'K55', 'N80', 'L80', 'C90', 'T95', 'P110', 'Q125'];

export default function PurchaseOrderFormPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm();

  const isEdit = !!id;
  const orderId = isEdit ? Number(id) : 0;

  const { data: order, isLoading: loadingOrder } = usePurchase(orderId);
  const createMutation = useCreatePurchaseOrder();
  const updateMutation = useUpdatePurchaseOrder(orderId);

  useEffect(() => {
    if (isEdit && order) {
      form.setFieldsValue({
        supplier_id: order.supplier_id,
        order_date: order.order_date ? dayjs(order.order_date) : undefined,
        expected_date: order.expected_date ? dayjs(order.expected_date) : undefined,
        notes: order.notes,
        items: order.items.map((item) => ({
          pipe_type: item.pipe_type,
          grade: item.grade,
          od: item.od,
          wt: item.wt,
          length: item.length,
          quantity: item.quantity,
          unit_price: item.unit_price,
          notes: item.notes,
        })),
      });
    }
  }, [isEdit, order, form]);

  const handleSubmit = async (values: Record<string, unknown>) => {
    const payload = {
      ...values,
      order_date: (values.order_date as dayjs.Dayjs)?.format('YYYY-MM-DD'),
      expected_date: (values.expected_date as dayjs.Dayjs)?.format('YYYY-MM-DD'),
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
      title: 'Pipe Type',
      dataIndex: 'pipe_type',
      key: 'pipe_type',
      width: 120,
      render: (_: unknown, __: unknown, index: number) => (
        <Form.Item
          name={['items', index, 'pipe_type']}
          rules={[{ required: true, message: t('common.required') }]}
          style={{ margin: 0 }}
        >
          <Select style={{ width: 120 }}>
            {PIPE_TYPES.map((type) => (
              <Select.Option key={type} value={type}>
                {type}
              </Select.Option>
            ))}
          </Select>
        </Form.Item>
      ),
    },
    {
      title: 'Grade',
      dataIndex: 'grade',
      key: 'grade',
      width: 100,
      render: (_: unknown, __: unknown, index: number) => (
        <Form.Item
          name={['items', index, 'grade']}
          rules={[{ required: true, message: t('common.required') }]}
          style={{ margin: 0 }}
        >
          <Select showSearch style={{ width: 100 }}>
            {API_5CT_GRADES.map((grade) => (
              <Select.Option key={grade} value={grade}>
                {grade}
              </Select.Option>
            ))}
          </Select>
        </Form.Item>
      ),
    },
    {
      title: 'OD (in)',
      dataIndex: 'od',
      key: 'od',
      width: 100,
      render: (_: unknown, __: unknown, index: number) => (
        <Form.Item
          name={['items', index, 'od']}
          rules={[{ required: true, message: t('common.required') }]}
          style={{ margin: 0 }}
        >
          <InputNumber min={0} step={0.001} style={{ width: '100%' }} />
        </Form.Item>
      ),
    },
    {
      title: 'WT (in)',
      dataIndex: 'wt',
      key: 'wt',
      width: 100,
      render: (_: unknown, __: unknown, index: number) => (
        <Form.Item
          name={['items', index, 'wt']}
          rules={[{ required: true, message: t('common.required') }]}
          style={{ margin: 0 }}
        >
          <InputNumber min={0} step={0.001} style={{ width: '100%' }} />
        </Form.Item>
      ),
    },
    {
      title: 'Length (m)',
      dataIndex: 'length',
      key: 'length',
      width: 100,
      render: (_: unknown, __: unknown, index: number) => (
        <Form.Item name={['items', index, 'length']} style={{ margin: 0 }}>
          <InputNumber min={0} step={0.01} style={{ width: '100%' }} />
        </Form.Item>
      ),
    },
    {
      title: 'Quantity',
      dataIndex: 'quantity',
      key: 'quantity',
      width: 100,
      render: (_: unknown, __: unknown, index: number) => (
        <Form.Item
          name={['items', index, 'quantity']}
          rules={[{ required: true, message: t('common.required') }]}
          style={{ margin: 0 }}
        >
          <InputNumber min={1} step={1} style={{ width: '100%' }} />
        </Form.Item>
      ),
    },
    {
      title: 'Unit Price',
      dataIndex: 'unit_price',
      key: 'unit_price',
      width: 120,
      render: (_: unknown, __: unknown, index: number) => (
        <Form.Item
          name={['items', index, 'unit_price']}
          rules={[{ required: true, message: t('common.required') }]}
          style={{ margin: 0 }}
        >
          <InputNumber min={0} step={0.01} prefix="$" style={{ width: '100%' }} />
        </Form.Item>
      ),
    },
    {
      title: 'Notes',
      dataIndex: 'notes',
      key: 'notes',
      width: 150,
      render: (_: unknown, __: unknown, index: number) => (
        <Form.Item name={['items', index, 'notes']} style={{ margin: 0 }}>
          <Input />
        </Form.Item>
      ),
    },
    {
      title: t('common.actions'),
      key: 'actions',
      width: 80,
      render: (_: unknown, __: unknown, index: number) => (
        <Popconfirm
          title="确认删除?"
          onConfirm={() => {
            const items = form.getFieldValue('items') || [];
            items.splice(index, 1);
            form.setFieldsValue({ items: [...items] });
          }}
        >
          <Button type="link" danger icon={<DeleteOutlined />} />
        </Popconfirm>
      ),
    },
  ];

  return (
    <div>
      <h2 style={{ marginBottom: 24 }}>
        {isEdit ? t('common.edit') : t('common.create')} Purchase Order
      </h2>
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        style={{ maxWidth: 1000 }}
      >
        <Form.Item
          label="Supplier ID"
          name="supplier_id"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <InputNumber min={1} style={{ width: 200 }} />
        </Form.Item>

        <Form.Item
          label="Order Date"
          name="order_date"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <DatePicker style={{ width: 200 }} />
        </Form.Item>

        <Form.Item label="Expected Date" name="expected_date">
          <DatePicker style={{ width: 200 }} />
        </Form.Item>

        <Form.Item label="Notes" name="notes">
          <Input.TextArea rows={3} style={{ maxWidth: 600 }} />
        </Form.Item>

        <h3 style={{ marginBottom: 16 }}>Items</h3>

        <Form.List name="items" initialValue={[]}>
          {(fields, { add, remove: _remove }) => (
            <>
              <Table
                columns={itemColumns}
                dataSource={fields.map((field) => ({ ...field }))}
                rowKey="key"
                pagination={false}
                footer={() => (
                  <Button
                    type="dashed"
                    onClick={() =>
                      add({
                        pipe_type: 'seamless',
                        grade: 'J55',
                        od: 0,
                        wt: 0,
                        length: undefined,
                        quantity: 1,
                        unit_price: 0,
                        notes: undefined,
                      })
                    }
                    icon={<PlusOutlined />}
                    style={{ width: '100%' }}
                  >
                    Add Item
                  </Button>
                )}
              />
            </>
          )}
        </Form.List>

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
    </div>
  );
}
