import React from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { Card, Descriptions, Tag, Table, Button, Spin, Space, Divider } from 'antd';
import { ArrowLeftOutlined, EditOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import { contractApi, Contract, ContractItem, ContractPayment } from '../api/contractApi';
import type { ColumnsType } from 'antd/es/table';

const statusConfig: Record<string, { color: string; label: string }> = {
  draft: { color: 'default', label: '草稿' },
  active: { color: 'blue', label: '生效中' },
  completed: { color: 'green', label: '已完成' },
  terminated: { color: 'orange', label: '已终止' },
  cancelled: { color: 'red', label: '已取消' },
};

export default function ContractDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const { data, isLoading } = useQuery({
    queryKey: ['contract', id],
    queryFn: () => contractApi.getContract(id!),
    enabled: !!id,
  });

  const contract = data?.data?.data;

  if (isLoading) return <Spin style={{ display: 'block', margin: '48px auto' }} />;
  if (!contract) return <Card>合同不存在</Card>;

  const itemColumns: ColumnsType<ContractItem> = [
    { title: '描述', dataIndex: 'description', key: 'description' },
    { title: '规格', dataIndex: 'spec', key: 'spec' },
    { title: '数量', dataIndex: 'quantity', key: 'quantity', align: 'right' },
    { title: '单价', dataIndex: 'unit_price', key: 'unit_price', align: 'right', render: (v: number) => `¥${v.toFixed(2)}` },
    { title: '金额', dataIndex: 'amount', key: 'amount', align: 'right', render: (v: number) => `¥${v.toFixed(2)}` },
    { title: '交货日期', dataIndex: 'delivery_date', key: 'delivery_date', render: (v?: string) => v ? dayjs(v).format('YYYY-MM-DD') : '-' },
  ];

  const paymentColumns: ColumnsType<ContractPayment> = [
    { title: '付款阶段', dataIndex: 'stage', key: 'stage' },
    { title: '金额', dataIndex: 'amount', key: 'amount', align: 'right', render: (v: number) => `¥${v.toFixed(2)}` },
    { title: '到期日', dataIndex: 'due_date', key: 'due_date', render: (v?: string) => v ? dayjs(v).format('YYYY-MM-DD') : '-' },
    { title: '付款日', dataIndex: 'paid_date', key: 'paid_date', render: (v?: string) => v ? dayjs(v).format('YYYY-MM-DD') : '-' },
    { title: '状态', dataIndex: 'status', key: 'status', render: (s: string) => <Tag>{s === 'paid' ? '已付款' : '待付款'}</Tag> },
  ];

  return (
    <Card
      title={<Space><Button icon={<ArrowLeftOutlined />} type="text" onClick={() => navigate('/contracts')} />合同详情</Space>}
      extra={<Button icon={<EditOutlined />} onClick={() => navigate(`/contracts/${id}/edit`)}>编辑</Button>}
      styles={{ body: { padding: 16 } }}
    >
      <Descriptions bordered column={2} size="small">
        <Descriptions.Item label="合同编号">{contract.contract_no}</Descriptions.Item>
        <Descriptions.Item label="合同类型">{contract.contract_type === 'sales' ? '销售合同' : '采购合同'}</Descriptions.Item>
        <Descriptions.Item label="对方名称">{contract.party_name}</Descriptions.Item>
        <Descriptions.Item label="总金额">¥{contract.total_amount?.toLocaleString()}</Descriptions.Item>
        <Descriptions.Item label="状态"><Tag color={statusConfig[contract.status]?.color}>{statusConfig[contract.status]?.label}</Tag></Descriptions.Item>
        <Descriptions.Item label="签订日期">{contract.sign_date ? dayjs(contract.sign_date).format('YYYY-MM-DD') : '-'}</Descriptions.Item>
        <Descriptions.Item label="生效日期">{contract.effective_date ? dayjs(contract.effective_date).format('YYYY-MM-DD') : '-'}</Descriptions.Item>
        <Descriptions.Item label="失效日期">{contract.expiry_date ? dayjs(contract.expiry_date).format('YYYY-MM-DD') : '-'}</Descriptions.Item>
        <Descriptions.Item label="备注" span={2}>{contract.notes || '-'}</Descriptions.Item>
      </Descriptions>

      <Divider>合同明细</Divider>
      <Table columns={itemColumns} dataSource={[]} rowKey="id" locale={{ emptyText: '暂无明细' }} pagination={false} />

      <Divider>付款计划</Divider>
      <Table columns={paymentColumns} dataSource={[]} rowKey="id" locale={{ emptyText: '暂无付款计划' }} pagination={false} />
    </Card>
  );
}
