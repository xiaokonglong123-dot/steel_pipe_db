import React, { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { Card, Form, Select, Button, Input, message, Spin } from 'antd';
import { labelApi } from '../api/labelApi';

export default function LabelGeneratePage() {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);

  const { data: templatesData, isLoading: templatesLoading } = useQuery({
    queryKey: ['label-templates', { page_size: 100 }],
    queryFn: () => labelApi.listTemplates({ page_size: 100 }),
  });

  const generateMutation = useMutation({
    mutationFn: (values: { template_id: string; pipe_ids: string[]; copies: number }) =>
      labelApi.printLabels(values),
    onSuccess: () => message.success('标签生成成功'),
    onError: () => message.error('生成失败'),
  });

  const onFinish = async (values: { template_id: string; pipe_numbers: string }) => {
    setLoading(true);
    try {
      const pipeIds = values.pipe_numbers.split('\n').map((s) => s.trim()).filter(Boolean);
      await generateMutation.mutateAsync({
        template_id: values.template_id,
        pipe_ids: pipeIds,
        copies: 1,
      });
    } finally {
      setLoading(false);
    }
  };

  const templates = templatesData?.data?.data || [];

  return (
    <Card title="生成标签" styles={{ body: { padding: 16 } }}>
      <Form form={form} layout="vertical" onFinish={onFinish} style={{ maxWidth: 600 }}>
        <Form.Item name="template_id" label="选择模板" rules={[{ required: true, message: '请选择模板' }]}>
          <Select
            placeholder="请选择标签模板"
            loading={templatesLoading}
            options={templates.map((t) => ({
              label: t.name,
              value: t.id,
            }))}
          />
        </Form.Item>
        <Form.Item name="pipe_numbers" label="管号列表（每行一个）" rules={[{ required: true, message: '请输入管号' }]}>
          <Input.TextArea rows={6} placeholder={'J55 4.500in×11.60lb SC-H2405-000001\nJ55 4.500in×11.60lb SC-H2405-000002'} />
        </Form.Item>
        <Form.Item>
          <Button type="primary" htmlType="submit" loading={loading}>
            生成标签
          </Button>
        </Form.Item>
      </Form>
    </Card>
  );
}
