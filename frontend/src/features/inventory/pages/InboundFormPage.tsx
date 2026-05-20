import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  Card,
  Form,
  Input,
  Select,
  Button,
  Space,
  message,
  Spin,
  Typography,
} from 'antd';
import { inventoryApi, PipeOption } from '../api/inventoryApi';

const { TextArea } = Input;
const { Text } = Typography;

const inboundTypeOptions = [
  { label: '采购入库', value: 'purchase' },
  { label: '退货入库', value: 'return' },
  { label: '调拨入库', value: 'transfer' },
];

export default function InboundFormPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const [pipeSearch, setPipeSearch] = useState('');

  const { data: pipesData, isLoading: pipesLoading } = useQuery({
    queryKey: ['all-pipes', pipeSearch],
    queryFn: () => inventoryApi.listAllPipes({ search: pipeSearch || undefined }),
  });

  const createMutation = useMutation({
    mutationFn: (values: { inbound_type: string; pipe_ids: string[]; notes?: string }) =>
      inventoryApi.createInbound(values),
    onSuccess: () => {
      message.success('入库创建成功');
      queryClient.invalidateQueries({ queryKey: ['inbound'] });
      queryClient.invalidateQueries({ queryKey: ['stock-summary'] });
      navigate('/inventory/inbound');
    },
    onError: () => {
      message.error('入库创建失败');
    },
  });

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields();
      createMutation.mutate(values);
    } catch {
      /* empty - validation errors shown by antd */
    }
  };

  const pipeOptions = (pipesData?.data?.data || []).map((p: PipeOption) => ({
    label: `${p.pipe_number} (${p.pipe_type === 'seamless' ? '无缝钢管' : '筛管'}, ${p.grade})`,
    value: p.id,
  }));

  return (
    <Card
      title="新增入库"
      extra={
        <Button onClick={() => navigate('/inventory/inbound')} type="default">
          返回列表
        </Button>
      }
      styles={{ body: { padding: 24 } }}
    >
      <Spin spinning={false}>
        <Form
          form={form}
          layout="vertical"
          initialValues={{ inbound_type: undefined, pipe_ids: [] }}
          style={{ maxWidth: 800 }}
        >
          <Form.Item
            name="inbound_type"
            label="入库类型"
            rules={[{ required: true, message: '请选择入库类型' }]}
          >
            <Select placeholder="请选择入库类型" options={inboundTypeOptions} />
          </Form.Item>

          <Form.Item
            name="pipe_ids"
            label="选择管材"
            rules={[{ required: true, message: '请至少选择一根管材' }]}
            extra="支持搜索管材编号，可多选"
          >
            <Select
              mode="multiple"
              placeholder="搜索并选择管材"
              showSearch
              filterOption={false}
              onSearch={setPipeSearch}
              loading={pipesLoading}
              options={pipeOptions}
              notFoundContent={pipesLoading ? <Spin size="small" /> : '无匹配管材'}
              style={{ width: '100%' }}
            />
          </Form.Item>

          <Text type="secondary" style={{ display: 'block', marginBottom: 16 }}>
            已选择 <Text strong>{form.getFieldValue('pipe_ids')?.length || 0}</Text> 根管材
          </Text>

          <Form.Item name="notes" label="备注">
            <TextArea rows={3} placeholder="可选备注信息" />
          </Form.Item>

          <Form.Item>
            <Space>
              <Button
                type="primary"
                onClick={handleSubmit}
                loading={createMutation.isPending}
              >
                创建入库
              </Button>
              <Button onClick={() => navigate('/inventory/inbound')}>取消</Button>
            </Space>
          </Form.Item>
        </Form>
      </Spin>
    </Card>
  );
}
