import React, { useState, useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useQuery, useMutation } from '@tanstack/react-query';
import { Card, Form, Input, InputNumber, Select, Switch, Button, message, Space, Spin } from 'antd';
import { labelApi, LabelTemplate } from '../api/labelApi';

export default function LabelTemplateFormPage() {
  const navigate = useNavigate();
  const { id } = useParams();
  const isEdit = !!id;
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);

  const { data: templateData, isLoading: queryLoading } = useQuery({
    queryKey: ['label-template', id],
    queryFn: () => labelApi.listTemplates(),
    enabled: isEdit,
  });

  useEffect(() => {
    if (isEdit && templateData?.data?.data) {
      const found = templateData.data.data.find((t: LabelTemplate) => t.id === id);
      if (found) form.setFieldsValue(found);
    }
  }, [templateData, id, isEdit, form]);

  const createMutation = useMutation({
    mutationFn: (values: Partial<LabelTemplate>) => labelApi.createTemplate(values),
    onSuccess: () => { message.success('创建成功'); navigate('/labels/templates'); },
    onError: () => message.error('创建失败'),
  });

  const updateMutation = useMutation({
    mutationFn: (values: Partial<LabelTemplate>) => labelApi.updateTemplate(id!, values),
    onSuccess: () => { message.success('更新成功'); navigate('/labels/templates'); },
    onError: () => message.error('更新失败'),
  });

  const onFinish = async (values: Record<string, unknown>) => {
    setLoading(true);
    try {
      if (isEdit) {
        await updateMutation.mutateAsync(values as Partial<LabelTemplate>);
      } else {
        await createMutation.mutateAsync(values as Partial<LabelTemplate>);
      }
    } finally {
      setLoading(false);
    }
  };

  if (isEdit && queryLoading) return <Spin style={{ display: 'block', margin: '48px auto' }} />;

  return (
    <Card title={isEdit ? '编辑标签模板' : '新增标签模板'} styles={{ body: { padding: 16 } }}>
      <Form form={form} layout="vertical" onFinish={onFinish} style={{ maxWidth: 600 }} initialValues={{ label_width: 100, label_height: 60, font_size: 12, is_default: false }}>
        <Form.Item name="template_name" label="模板名称" rules={[{ required: true, message: '请输入模板名称' }]}>
          <Input placeholder="请输入模板名称" />
        </Form.Item>
        <Form.Item name="pipe_type" label="管材类型" rules={[{ required: true, message: '请选择管材类型' }]}>
          <Select options={[{ label: '无缝钢管', value: 'seamless' }, { label: '筛管', value: 'screen' }]} placeholder="请选择" />
        </Form.Item>
        <Space size={16}>
          <Form.Item name="label_width" label="标签宽度(mm)" rules={[{ required: true }]}>
            <InputNumber min={20} max={300} />
          </Form.Item>
          <Form.Item name="label_height" label="标签高度(mm)" rules={[{ required: true }]}>
            <InputNumber min={10} max={200} />
          </Form.Item>
        </Space>
        <Form.Item name="font_size" label="字体大小" rules={[{ required: true }]}>
          <InputNumber min={6} max={72} />
        </Form.Item>
        <Form.Item name="fields_config" label="字段配置(JSON)">
          <Input.TextArea rows={4} placeholder='["pipe_number","grade","od","wt","heat_number"]' />
        </Form.Item>
        <Form.Item name="is_default" label="设为默认模板" valuePropName="checked">
          <Switch />
        </Form.Item>
        <Form.Item>
          <Space>
            <Button type="primary" htmlType="submit" loading={loading}>{isEdit ? '更新' : '创建'}</Button>
            <Button onClick={() => navigate('/labels/templates')}>取消</Button>
          </Space>
        </Form.Item>
      </Form>
    </Card>
  );
}
