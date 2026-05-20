import React, { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
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
  Modal,
  Tag,
} from 'antd';
import { PlusOutlined, MinusCircleOutlined } from '@ant-design/icons';
import { salesApi, AtpResult } from '../api/salesApi';
import CustomerSelect from '../components/CustomerSelect';
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

const pipeTypeLabelMap: Record<string, string> = {
  seamless: '无缝钢管',
  screen: '筛管',
};

interface OrderItemRecord {
  key: string;
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  quantity: number;
  unit_price: number;
}

export default function SalesOrderFormPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const isEdit = Boolean(id);

  const [atpModalOpen, setAtpModalOpen] = useState(false);
  const [atpResults, setAtpResults] = useState<AtpResult[]>([]);
  const [atpLoading, setAtpLoading] = useState(false);

  const title = isEdit ? '编辑销售订单' : '新增销售订单';
  const listPath = '/sales';

  const handleATPCheck = async () => {
    try {
      const items = form.getFieldValue('items') as OrderItemRecord[];
      if (!items || items.length === 0) {
        message.warning('请先添加订单明细');
        return;
      }
      const incomplete = items.some(
        (item) => !item.pipe_type || !item.grade || !item.od || !item.wt || !item.quantity,
      );
      if (incomplete) {
        message.warning('请先完善订单明细中的必填项');
        return;
      }
      setAtpLoading(true);
      const results = await Promise.all(
        items.map((item) =>
          salesApi
            .checkAtp({
              pipe_type: item.pipe_type,
              grade: item.grade,
              od: item.od,
              wt: item.wt,
              quantity: item.quantity,
            })
            .then((r) => r.data.data!),
        ),
      );
      setAtpResults(results);
      setAtpModalOpen(true);
    } catch {
      message.error('ATP查询失败');
    } finally {
      setAtpLoading(false);
    }
  };

  const { data: orderData, isLoading: orderLoading } = useQuery({
    queryKey: ['sales-orders', id],
    queryFn: () => salesApi.get(id!),
    enabled: isEdit,
  });

  useEffect(() => {
    if (!orderData?.data?.data) return;
    const order = orderData.data.data as unknown as Record<string, unknown>;
    form.setFieldsValue({
      customer_id: order.customer_id,
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

      return salesApi.create({
        customer_id: values.customer_id as string,
        notes: values.notes as string | undefined,
        items: items as Parameters<typeof salesApi.create>[0]['items'],
      }) as Promise<unknown>;
    },
    onSuccess: () => {
      message.success('销售订单创建成功');
      queryClient.invalidateQueries({ queryKey: ['sales-orders'] });
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
              name="customer_id"
              label="客户"
              rules={[{ required: true, message: '请选择客户' }]}
            >
              <CustomerSelect disabled={isEdit} />
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
                      <Button onClick={handleATPCheck} loading={atpLoading}>
                        查询可售库存
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

        {/* ATP Result Modal */}
        <Modal
          title="可售库存查询结果"
          open={atpModalOpen}
          onCancel={() => setAtpModalOpen(false)}
          footer={null}
          width={680}
        >
          {atpResults.length === 0 ? (
            <div style={{ textAlign: 'center', padding: 24, color: '#999' }}>
              暂无结果
            </div>
          ) : (
            atpResults.map((result, index) => {
              const label = `${pipeTypeLabelMap[result.pipe_type] || result.pipe_type} / ${result.grade} / ${result.od.toFixed(3)}in × ${result.wt.toFixed(3)}in`;
              return (
                <div
                  key={index}
                  style={{
                    marginBottom: 12,
                    padding: 12,
                    border: '1px solid #f0f0f0',
                    borderRadius: 6,
                    background: result.available ? '#f6ffed' : '#fff2f0',
                  }}
                >
                  <div style={{ marginBottom: 8, fontWeight: 500 }}>{label}</div>
                  <Space>
                    <Tag color={result.available ? 'green' : 'red'}>
                      {result.available ? '充足' : '不足'}
                    </Tag>
                    <span>
                      可售库存：<strong>{result.available_qty}</strong>
                      {' / '}
                      需求：<strong>{result.requested_qty}</strong>
                    </span>
                  </Space>
                </div>
              );
            })
          )}
        </Modal>

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
