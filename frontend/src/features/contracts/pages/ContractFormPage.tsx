import React, { useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Card,
  Form,
  Input,
  InputNumber,
  Select,
  DatePicker,
  Button,
  Space,
  Row,
  Col,
  message,
  Spin,
  Divider,
} from 'antd';
import { PlusOutlined, MinusCircleOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import { contractApi } from '../api/contractApi';

const { TextArea } = Input;
const { Item: FormItem } = Form;

const contractTypeOptions = [
  { label: '销售合同', value: 'sales' },
  { label: '采购合同', value: 'purchase' },
];

const paymentStatusOptions = [
  { label: '待付款', value: 'pending' },
  { label: '已付款', value: 'paid' },
  { label: '逾期', value: 'overdue' },
];

export default function ContractFormPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const isEdit = Boolean(id);

  const { data: editData, isLoading: loadingEdit } = useQuery({
    queryKey: ['contract-edit', id],
    queryFn: () => contractApi.getContract(id!),
    enabled: isEdit,
  });

  useEffect(() => {
    if (editData?.data?.data) {
      const record = editData.data.data;
      form.setFieldsValue({
        ...record,
        sign_date: record.sign_date ? dayjs(record.sign_date) : undefined,
        effective_date: record.effective_date ? dayjs(record.effective_date) : undefined,
        expiry_date: record.expiry_date ? dayjs(record.expiry_date) : undefined,
      });
    }
  }, [editData, form]);

  const createMutation = useMutation({
    mutationFn: (values: Record<string, unknown>) => {
      const payload = { ...values };
      ['sign_date', 'effective_date', 'expiry_date'].forEach((key) => {
        if (payload[key]) {
          payload[key] = dayjs(payload[key] as string).format('YYYY-MM-DD');
        }
      });
      return contractApi.createContract(payload);
    },
    onSuccess: () => {
      message.success('合同创建成功');
      queryClient.invalidateQueries({ queryKey: ['contracts'] });
      navigate('/contracts');
    },
    onError: () => {
      message.error('创建失败');
    },
  });

  const updateMutation = useMutation({
    mutationFn: (values: Record<string, unknown>) => {
      const payload = { ...values };
      ['sign_date', 'effective_date', 'expiry_date'].forEach((key) => {
        if (payload[key]) {
          payload[key] = dayjs(payload[key] as string).format('YYYY-MM-DD');
        }
      });
      return contractApi.updateContract(id!, payload);
    },
    onSuccess: () => {
      message.success('合同更新成功');
      queryClient.invalidateQueries({ queryKey: ['contracts'] });
      navigate('/contracts');
    },
    onError: () => {
      message.error('更新失败');
    },
  });

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields();
      if (isEdit) {
        updateMutation.mutate(values);
      } else {
        createMutation.mutate(values);
      }
    } catch {
      /* empty - validation errors shown by antd */
    }
  };

  const isSubmitting = createMutation.isPending || updateMutation.isPending;

  return (
    <Card
      title={isEdit ? '编辑合同' : '新增合同'}
      extra={
        <Button onClick={() => navigate('/contracts')} type="default">
          返回列表
        </Button>
      }
      styles={{ body: { padding: 24 } }}
    >
      <Spin spinning={isEdit && loadingEdit}>
        <Form
          form={form}
          layout="vertical"
          initialValues={{ status: 'draft' }}
          style={{ maxWidth: 900 }}
        >
          <Row gutter={24}>
            <Col xs={24} sm={12} md={8}>
              <FormItem
                name="contract_no"
                label="合同编号"
                rules={[{ required: true, message: '请输入合同编号' }]}
              >
                <Input placeholder="例如: CT-2025-0001" />
              </FormItem>
            </Col>
            <Col xs={24} sm={12} md={8}>
              <FormItem
                name="contract_type"
                label="合同类型"
                rules={[{ required: true, message: '请选择合同类型' }]}
              >
                <Select placeholder="请选择合同类型" options={contractTypeOptions} />
              </FormItem>
            </Col>
            <Col xs={24} sm={12} md={8}>
              <FormItem
                name="party_name"
                label="对方名称"
                rules={[{ required: true, message: '请输入对方名称' }]}
              >
                <Input placeholder="客户/供应商名称" />
              </FormItem>
            </Col>
          </Row>

          <Row gutter={24}>
            <Col xs={12} sm={8} md={6}>
              <FormItem
                name="total_amount"
                label="总金额"
                rules={[{ required: true, message: '请输入总金额' }]}
              >
                <InputNumber
                  style={{ width: '100%' }}
                  min={0}
                  precision={2}
                  prefix="¥"
                  placeholder="0.00"
                />
              </FormItem>
            </Col>
            <Col xs={12} sm={8} md={6}>
              <FormItem name="sign_date" label="签订日期">
                <DatePicker style={{ width: '100%' }} />
              </FormItem>
            </Col>
            <Col xs={12} sm={8} md={6}>
              <FormItem name="effective_date" label="生效日期">
                <DatePicker style={{ width: '100%' }} />
              </FormItem>
            </Col>
            <Col xs={12} sm={8} md={6}>
              <FormItem name="expiry_date" label="到期日期">
                <DatePicker style={{ width: '100%' }} />
              </FormItem>
            </Col>
          </Row>

          <Row gutter={24}>
            <Col xs={24}>
              <FormItem name="notes" label="备注">
                <TextArea rows={3} placeholder="可选备注信息" />
              </FormItem>
            </Col>
          </Row>

          <Divider orientation="left" plain>合同明细</Divider>

          <Form.List name="items" initialValue={[]}>
            {(fields, { add, remove }) => (
              <>
                <div style={{ overflowX: 'auto' }}>
                  <table style={{ width: '100%', minWidth: 700, borderCollapse: 'collapse' }}>
                    <thead>
                      <tr style={{ borderBottom: '1px solid #f0f0f0' }}>
                        <th style={{ padding: '8px 4px', textAlign: 'left', width: 30 }}>#</th>
                        <th style={{ padding: '8px 4px', textAlign: 'left', width: '25%' }}>描述</th>
                        <th style={{ padding: '8px 4px', textAlign: 'left', width: '15%' }}>规格</th>
                        <th style={{ padding: '8px 4px', textAlign: 'right', width: '12%' }}>数量</th>
                        <th style={{ padding: '8px 4px', textAlign: 'right', width: '14%' }}>单价</th>
                        <th style={{ padding: '8px 4px', textAlign: 'right', width: '14%' }}>金额</th>
                        <th style={{ padding: '8px 4px', textAlign: 'left', width: '14%' }}>交货日期</th>
                        <th style={{ padding: '8px 4px', width: 40 }} />
                      </tr>
                    </thead>
                    <tbody>
                      {fields.map(({ key, name, ...rest }) => (
                        <tr key={key} style={{ borderBottom: '1px solid #f0f0f0' }}>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            {name + 1}
                          </td>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            <FormItem
                              {...rest}
                              name={[name, 'description']}
                              rules={[{ required: true, message: '请输入描述' }]}
                              noStyle
                            >
                              <Input placeholder="描述" />
                            </FormItem>
                          </td>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            <FormItem {...rest} name={[name, 'spec']} noStyle>
                              <Input placeholder="规格" />
                            </FormItem>
                          </td>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            <FormItem
                              {...rest}
                              name={[name, 'quantity']}
                              rules={[{ required: true, message: '请输入数量' }]}
                              noStyle
                            >
                              <InputNumber style={{ width: '100%' }} min={0} placeholder="数量" />
                            </FormItem>
                          </td>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            <FormItem
                              {...rest}
                              name={[name, 'unit_price']}
                              rules={[{ required: true, message: '请输入单价' }]}
                              noStyle
                            >
                              <InputNumber
                                style={{ width: '100%' }}
                                min={0}
                                precision={2}
                                prefix="¥"
                                placeholder="单价"
                              />
                            </FormItem>
                          </td>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            <FormItem
                              {...rest}
                              name={[name, 'amount']}
                              rules={[{ required: true, message: '请输入金额' }]}
                              noStyle
                            >
                              <InputNumber
                                style={{ width: '100%' }}
                                min={0}
                                precision={2}
                                prefix="¥"
                                placeholder="金额"
                              />
                            </FormItem>
                          </td>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            <FormItem {...rest} name={[name, 'delivery_date']} noStyle>
                              <DatePicker style={{ width: '100%' }} />
                            </FormItem>
                          </td>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            <MinusCircleOutlined
                              style={{ color: '#ff4d4f', cursor: 'pointer', marginTop: 8 }}
                              onClick={() => remove(name)}
                            />
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                <Button
                  type="dashed"
                  onClick={() => add()}
                  block
                  icon={<PlusOutlined />}
                  style={{ marginTop: 8 }}
                >
                  添加明细
                </Button>
              </>
            )}
          </Form.List>

          <Divider orientation="left" plain>付款计划</Divider>

          <Form.List name="payments" initialValue={[]}>
            {(fields, { add, remove }) => (
              <>
                <div style={{ overflowX: 'auto' }}>
                  <table style={{ width: '100%', minWidth: 600, borderCollapse: 'collapse' }}>
                    <thead>
                      <tr style={{ borderBottom: '1px solid #f0f0f0' }}>
                        <th style={{ padding: '8px 4px', textAlign: 'left', width: 30 }}>#</th>
                        <th style={{ padding: '8px 4px', textAlign: 'left', width: '20%' }}>付款阶段</th>
                        <th style={{ padding: '8px 4px', textAlign: 'right', width: '18%' }}>金额</th>
                        <th style={{ padding: '8px 4px', textAlign: 'left', width: '18%' }}>到期日</th>
                        <th style={{ padding: '8px 4px', textAlign: 'left', width: '18%' }}>付款日期</th>
                        <th style={{ padding: '8px 4px', textAlign: 'left', width: '14%' }}>状态</th>
                        <th style={{ padding: '8px 4px', width: 40 }} />
                      </tr>
                    </thead>
                    <tbody>
                      {fields.map(({ key, name, ...rest }) => (
                        <tr key={key} style={{ borderBottom: '1px solid #f0f0f0' }}>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            {name + 1}
                          </td>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            <FormItem
                              {...rest}
                              name={[name, 'stage']}
                              rules={[{ required: true, message: '请输入付款阶段' }]}
                              noStyle
                            >
                              <Input placeholder="例如: 首付款" />
                            </FormItem>
                          </td>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            <FormItem
                              {...rest}
                              name={[name, 'amount']}
                              rules={[{ required: true, message: '请输入金额' }]}
                              noStyle
                            >
                              <InputNumber
                                style={{ width: '100%' }}
                                min={0}
                                precision={2}
                                prefix="¥"
                                placeholder="金额"
                              />
                            </FormItem>
                          </td>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            <FormItem {...rest} name={[name, 'due_date']} noStyle>
                              <DatePicker style={{ width: '100%' }} />
                            </FormItem>
                          </td>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            <FormItem {...rest} name={[name, 'paid_date']} noStyle>
                              <DatePicker style={{ width: '100%' }} />
                            </FormItem>
                          </td>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            <FormItem
                              {...rest}
                              name={[name, 'status']}
                              rules={[{ required: true, message: '请选择状态' }]}
                              noStyle
                            >
                              <Select placeholder="状态" options={paymentStatusOptions} />
                            </FormItem>
                          </td>
                          <td style={{ padding: '8px 4px', verticalAlign: 'top' }}>
                            <MinusCircleOutlined
                              style={{ color: '#ff4d4f', cursor: 'pointer', marginTop: 8 }}
                              onClick={() => remove(name)}
                            />
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                <Button
                  type="dashed"
                  onClick={() => add()}
                  block
                  icon={<PlusOutlined />}
                  style={{ marginTop: 8 }}
                >
                  添加付款阶段
                </Button>
              </>
            )}
          </Form.List>

          <Form.Item style={{ marginTop: 24 }}>
            <Space>
              <Button type="primary" onClick={handleSubmit} loading={isSubmitting}>
                {isEdit ? '保存修改' : '创建'}
              </Button>
              <Button onClick={() => navigate('/contracts')}>取消</Button>
            </Space>
          </Form.Item>
        </Form>
      </Spin>
    </Card>
  );
}
