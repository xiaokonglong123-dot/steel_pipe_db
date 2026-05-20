import React, { useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Card,
  Form,
  Input,
  Switch,
  Button,
  Space,
  Row,
  Col,
  message,
  Spin,
} from 'antd';
import { customerApi } from '../api/customerApi';

const { TextArea } = Input;

export default function CustomerFormPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const isEdit = Boolean(id);

  const { data: editData, isLoading: loadingEdit } = useQuery({
    queryKey: ['customer-edit', id],
    queryFn: () => customerApi.get(id!),
    enabled: isEdit,
  });

  useEffect(() => {
    if (editData?.data?.data) {
      form.setFieldsValue(editData.data.data);
    }
  }, [editData, form]);

  const createMutation = useMutation({
    mutationFn: (values: Record<string, unknown>) =>
      customerApi.create(values as unknown as Parameters<typeof customerApi.create>[0]),
    onSuccess: () => {
      message.success('客户创建成功');
      queryClient.invalidateQueries({ queryKey: ['customers'] });
      navigate('/sales/customers');
    },
    onError: () => {
      message.error('创建失败');
    },
  });

  const updateMutation = useMutation({
    mutationFn: (values: Record<string, unknown>) =>
      customerApi.update(id!, values as unknown as Parameters<typeof customerApi.update>[1]),
    onSuccess: () => {
      message.success('客户更新成功');
      queryClient.invalidateQueries({ queryKey: ['customers'] });
      navigate('/sales/customers');
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
      // validation errors handled by form
    }
  };

  const isSubmitting = createMutation.isPending || updateMutation.isPending;

  return (
    <Card
      title={isEdit ? '编辑客户' : '新增客户'}
      extra={
        <Button onClick={() => navigate('/sales/customers')} type="default">
          返回列表
        </Button>
      }
      styles={{ body: { padding: 24 } }}
    >
      <Spin spinning={isEdit && loadingEdit}>
        <Form
          form={form}
          layout="vertical"
          initialValues={{ is_active: true }}
          style={{ maxWidth: 800 }}
        >
          <Row gutter={24}>
            <Col xs={24} sm={12} md={8}>
              <Form.Item
                name="name"
                label="客户名称"
                rules={[{ required: true, message: '请输入客户名称' }]}
              >
                <Input placeholder="客户名称" />
              </Form.Item>
            </Col>
            <Col xs={24} sm={12} md={8}>
              <Form.Item name="contact_person" label="联系人">
                <Input placeholder="联系人姓名" />
              </Form.Item>
            </Col>
            <Col xs={24} sm={12} md={8}>
              <Form.Item name="phone" label="电话">
                <Input placeholder="联系电话" />
              </Form.Item>
            </Col>
          </Row>

          <Row gutter={24}>
            <Col xs={24} sm={12} md={8}>
              <Form.Item name="email" label="邮箱">
                <Input placeholder="电子邮箱" />
              </Form.Item>
            </Col>
            <Col xs={24} sm={12} md={8}>
              <Form.Item name="is_active" label="状态" valuePropName="checked">
                <Switch checkedChildren="启用" unCheckedChildren="停用" />
              </Form.Item>
            </Col>
          </Row>

          <Row gutter={24}>
            <Col xs={24}>
              <Form.Item name="address" label="地址">
                <Input placeholder="详细地址" />
              </Form.Item>
            </Col>
          </Row>

          <Form.Item>
            <Space>
              <Button type="primary" onClick={handleSubmit} loading={isSubmitting}>
                {isEdit ? '保存修改' : '创建'}
              </Button>
              <Button onClick={() => navigate('/sales/customers')}>取消</Button>
            </Space>
          </Form.Item>
        </Form>
      </Spin>
    </Card>
  );
}
