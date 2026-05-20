import React, { useEffect, useState } from 'react';
import { useNavigate, useParams, useLocation } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Card,
  Form,
  Input,
  Select,
  Button,
  Space,
  Row,
  Col,
  Table,
  message,
  InputNumber,
  Spin,
} from 'antd';
import { PlusOutlined, MinusCircleOutlined } from '@ant-design/icons';
import { purchaseApi } from '../api/purchaseApi';
import SupplierSelect from '../components/SupplierSelect';
import type { ColumnsType } from 'antd/es/table';

const { TextArea } = Input;

const gradeOptions = [
  { label: 'J55', value: 'J55' },
  { label: 'K55', value: 'K55' },
  { label: 'N80', value: 'N80' },
  { label: 'L80', value: 'L80' },
  { label: 'P110', value: 'P110' },
];

const pipeTypeOptions = [
  { label: '无缝钢管', value: 'seamless' },
  { label: '筛管', value: 'screen' },
];

interface OrderItemRecord {
  key: string;
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  quantity: number;
  unit_price: number;
}

export default function PurchaseOrderFormPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const isEdit = Boolean(id);

  const title = isEdit ? '编辑采购订单' : '新增采购订单';
  const listPath = '/purchases';

  const { data: orderData, isLoading: orderLoading } = useQuery({
    queryKey: ['purchase-orders', id],
    queryFn: () => purchaseApi.get(id!),
    enabled: isEdit,
  });

  useEffect(() => {
    if (!orderData?.data?.data) return;
    const order = orderData.data.data as unknown as Record<string, unknown>;
    form.setFieldsValue({
      supplier_id: order.supplier_id,
      notes: order.notes as string | undefined,
      items: (order.items as Record<string, unknown>[])?.map((item) => ({
        pipe_type: item.pipe_type,
        grade: item.grade,
        od: item.od,
        wt: item.wt,
        quantity: item.quantity,
        unit_price: item.unit_price,
      })) || [{}],
    });
  }, [orderData, form]);

  const createMutation = useMutation({
    mutationFn: (values: Record<string, unknown>) => {
      const items = (values.items as OrderItemRecord[]).map((item) => ({
        pipe_type: item.pipe_type,
        grade: item.grade,
        od: item.od,
        wt: item.wt,
        quantity: item.quantity,
        unit_price: item.unit_price,
      }));

      return purchaseApi.create({
        supplier_id: values.supplier_id as string,
        notes: values.notes as string | undefined,
        items: items as Parameters<typeof purchaseApi.create>[0]['items'],
      }) as Promise<unknown>;
    },
    onSuccess: () => {
      message.success('采购订单创建成功');
      queryClient.invalidateQueries({ queryKey: ['purchase-orders'] });
      navigate(listPath);
    },
    onError: () => {
      message.error('创建失败');
    },
  });

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields();
      createMutation.mutate(values);
    } catch {
      // validation errors handled by form
    }
  };

  const itemColumns: ColumnsType<{ key: string }> = [
    {
      title: '管材类型',
      dataIndex: 'pipe_type',
      width: 140,
      render: (_, __, index) => (
        <Form.Item
          name={['items', index, 'pipe_type']}
          rules={[{ required: true, message: '请选择' }]}
          style={{ margin: 0 }}
        >
          <Select placeholder="类型" options={pipeTypeOptions} style={{ width: 120 }} />
        </Form.Item>
      ),
    },
    {
      title: '钢级',
      dataIndex: 'grade',
      width: 110,
      render: (_, __, index) => (
        <Form.Item
          name={['items', index, 'grade']}
          rules={[{ required: true, message: '请选择' }]}
          style={{ margin: 0 }}
        >
          <Select placeholder="钢级" options={gradeOptions} style={{ width: 90 }} />
        </Form.Item>
      ),
    },
    {
      title: '外径(in)',
      dataIndex: 'od',
      width: 110,
      render: (_, __, index) => (
        <Form.Item
          name={['items', index, 'od']}
          rules={[{ required: true, message: '请输入' }]}
          style={{ margin: 0 }}
        >
          <InputNumber step={0.001} placeholder="4.500" style={{ width: 90 }} />
        </Form.Item>
      ),
    },
    {
      title: '壁厚(in)',
      dataIndex: 'wt',
      width: 110,
      render: (_, __, index) => (
        <Form.Item
          name={['items', index, 'wt']}
          rules={[{ required: true, message: '请输入' }]}
          style={{ margin: 0 }}
        >
          <InputNumber step={0.001} placeholder="0.250" style={{ width: 90 }} />
        </Form.Item>
      ),
    },
    {
      title: '数量',
      dataIndex: 'quantity',
      width: 100,
      render: (_, __, index) => (
        <Form.Item
          name={['items', index, 'quantity']}
          rules={[{ required: true, message: '请输入' }]}
          style={{ margin: 0 }}
        >
          <InputNumber min={1} placeholder="1" style={{ width: 80 }} />
        </Form.Item>
      ),
    },
    {
      title: '单价',
      dataIndex: 'unit_price',
      width: 120,
      render: (_, __, index) => (
        <Form.Item
          name={['items', index, 'unit_price']}
          rules={[{ required: true, message: '请输入' }]}
          style={{ margin: 0 }}
        >
          <InputNumber min={0} step={0.01} prefix="¥" placeholder="0.00" style={{ width: 100 }} />
        </Form.Item>
      ),
    },
    {
      title: '操作',
      width: 60,
      render: (_, __, index) => (
        <MinusCircleOutlined
          onClick={() => {
            const currentItems = form.getFieldValue('items') || [];
            if (currentItems.length > 1) {
              const updated = currentItems.filter((_: unknown, i: number) => i !== index);
              form.setFieldValue('items', updated);
            }
          }}
        />
      ),
    },
  ];

  if (isEdit && orderLoading) {
    return (
      <Card title={title}>
        <div style={{ textAlign: 'center', padding: 48 }}>
          <Spin size="large" />
        </div>
      </Card>
    );
  }

  return (
    <Card
      title={title}
      extra={
        <Button onClick={() => navigate(listPath)} type="default">
          返回列表
        </Button>
      }
      styles={{ body: { padding: 24 } }}
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={{ items: [{}] }}
        style={{ maxWidth: 960 }}
      >
        <Row gutter={24}>
          <Col xs={24} sm={12} md={8}>
            <Form.Item
              name="supplier_id"
              label="供应商"
              rules={[{ required: true, message: '请选择供应商' }]}
            >
              <SupplierSelect disabled={isEdit} />
            </Form.Item>
          </Col>
        </Row>

        <Row gutter={24} style={{ marginTop: 16, marginBottom: 8 }}>
          <Col xs={24}>
            <strong>订单明细</strong>
          </Col>
        </Row>

        <Form.List name="items">
          {(fields, { add }) => (
            <>
              <Table
                dataSource={fields.map((f) => ({ key: String(f.name) }))}
                columns={itemColumns}
                rowKey="key"
                pagination={false}
                locale={{ emptyText: '暂无明细' }}
                scroll={{ x: 760 }}
                footer={() =>
                  !isEdit ? (
                    <Space style={{ width: '100%' }}>
                      <Button
                        type="dashed"
                        onClick={() => add({} as OrderItemRecord)}
                        icon={<PlusOutlined />}
                      >
                        添加行
                      </Button>
                    </Space>
                  ) : null
                }
              />
              {fields.length === 0 && (
                <div style={{ textAlign: 'center', padding: 24, color: '#999' }}>
                  请点击"添加行"增加订单明细
                </div>
              )}
            </>
          )}
        </Form.List>

        <Row gutter={24} style={{ marginTop: 16 }}>
          <Col xs={24}>
            <Form.Item name="notes" label="备注">
              <TextArea rows={3} placeholder="可选备注信息" />
            </Form.Item>
          </Col>
        </Row>

        <Form.Item>
          <Space>
            {!isEdit && (
              <Button type="primary" onClick={handleSubmit} loading={createMutation.isPending}>
                创建订单
              </Button>
            )}
            <Button onClick={() => navigate(listPath)}>取消</Button>
          </Space>
        </Form.Item>
      </Form>
    </Card>
  );
}
