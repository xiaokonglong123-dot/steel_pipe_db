import React from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Card,
  Descriptions,
  Table,
  Button,
  Spin,
  Space,
  Steps,
  Popconfirm,
  Divider,
  message,
} from 'antd';
import { ArrowLeftOutlined, EditOutlined } from '@ant-design/icons';
import { purchaseApi } from '../api/purchaseApi';
import OrderStatusTag from '../components/OrderStatusTag';
import type { InboundRef } from '../api/purchaseApi';
import type { ColumnsType } from 'antd/es/table';
import type { OrderStatus } from '../../../shared/types';

const pipeTypeLabels: Record<string, string> = {
  seamless: '无缝钢管',
  screen: '筛管',
};

const stepMapping: Record<string, number> = {
  draft: 0,
  pending: 1,
  approved: 2,
  completed: 3,
};

const stepItems = [
  { title: '草稿' },
  { title: '待审核' },
  { title: '已审核' },
  { title: '已完成' },
];

interface OrderItemDisplay {
  key: string;
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  quantity: number;
  unit_price: number;
  subtotal?: number;
}

export default function PurchaseOrderDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const { data, isLoading } = useQuery({
    queryKey: ['purchase-orders', id],
    queryFn: () => purchaseApi.get(id!),
    enabled: !!id,
  });

  const order = data?.data?.data as Record<string, unknown> | undefined;
  const status = (order?.status as OrderStatus) || '';
  const orderItems = (order?.items as OrderItemDisplay[]) || [];
  const inboundRefs = (order?.inbound_refs as InboundRef[]) || [];

  const cancelMutation = useMutation({
    mutationFn: async () => {
      await purchaseApi.cancel(id!);
    },
    onSuccess: () => {
      message.success('订单已取消');
      queryClient.invalidateQueries({ queryKey: ['purchase-orders', id] });
    },
    onError: () => {
      message.error('取消失败');
    },
  });

  const approveMutation = useMutation({
    mutationFn: async () => {
      await purchaseApi.approve(id!);
    },
    onSuccess: () => {
      message.success('订单已审核通过');
      queryClient.invalidateQueries({ queryKey: ['purchase-orders', id] });
    },
    onError: () => {
      message.error('审核失败');
    },
  });

  if (isLoading) {
    return <Spin style={{ display: 'block', margin: '48px auto' }} />;
  }

  if (!order) {
    return <Card>订单不存在</Card>;
  }

  const supplierName = (order.supplier_name as string) || '-';
  const operatorName = (order.operator_name as string) || '-';
  const orderNo = (order.order_no as string) || '';
  const createdAt = (order.created_at as string) || '';
  const totalAmount = (order.total_amount as number) || 0;
  const notes = (order.notes as string) || '';
  const stepCurrent = stepMapping[status] ?? 0;
  const isCancelled = status === 'cancelled';

  const itemColumns: ColumnsType<OrderItemDisplay> = [
    {
      title: '管材类型',
      dataIndex: 'pipe_type',
      width: 120,
      render: (v: string) => pipeTypeLabels[v] || v,
    },
    { title: '钢级', dataIndex: 'grade', width: 80 },
    {
      title: '外径(in)',
      dataIndex: 'od',
      width: 100,
      render: (v: number) => v.toFixed(3),
    },
    {
      title: '壁厚(in)',
      dataIndex: 'wt',
      width: 100,
      render: (v: number) => v.toFixed(3),
    },
    { title: '数量', dataIndex: 'quantity', width: 80, align: 'right' },
    {
      title: '单价',
      dataIndex: 'unit_price',
      width: 100,
      align: 'right',
      render: (v: number) => `¥${v.toFixed(2)}`,
    },
    {
      title: '小计',
      dataIndex: 'subtotal',
      width: 100,
      align: 'right',
      render: (v: number | undefined) =>
        v != null ? `¥${v.toFixed(2)}` : '-',
    },
  ];

  const editUrl = `/purchases/${id}/edit`;

  return (
    <Card
      title={
        <Space>
          <Button
            icon={<ArrowLeftOutlined />}
            type="text"
            onClick={() => navigate('/purchases')}
          />
          采购订单详情
        </Space>
      }
      extra={
        <Button onClick={() => navigate(editUrl)} icon={<EditOutlined />}>
          编辑
        </Button>
      }
      styles={{ body: { padding: 16 } }}
    >
      <Descriptions bordered column={2} size="small">
        <Descriptions.Item label="订单编号">{orderNo}</Descriptions.Item>
        <Descriptions.Item label="订单类型">采购订单</Descriptions.Item>
        <Descriptions.Item label="供应商">{supplierName}</Descriptions.Item>
        <Descriptions.Item label="操作人">{operatorName}</Descriptions.Item>
        <Descriptions.Item label="总金额">
          ¥{totalAmount.toLocaleString(undefined, { minimumFractionDigits: 2 })}
        </Descriptions.Item>
        <Descriptions.Item label="创建时间">{createdAt}</Descriptions.Item>
        <Descriptions.Item label="备注" span={2}>
          {notes || '-'}
        </Descriptions.Item>
      </Descriptions>

      <Divider>订单状态</Divider>
      <div style={{ marginBottom: 16 }}>
        <OrderStatusTag status={status} />
      </div>
      <Steps
        current={isCancelled ? 0 : stepCurrent}
        status={isCancelled ? 'error' : undefined}
        items={stepItems}
        style={{ maxWidth: 600, marginBottom: 16 }}
      />
      <Space>
        {(status === 'draft' || status === 'pending') && (
          <Popconfirm
            title="确定要取消此订单吗？"
            onConfirm={() => cancelMutation.mutate()}
            okText="确定"
            cancelText="取消"
          >
            <Button danger loading={cancelMutation.isPending}>
              取消订单
            </Button>
          </Popconfirm>
        )}
        {status === 'pending' && (
          <Button
            type="primary"
            loading={approveMutation.isPending}
            onClick={() => approveMutation.mutate()}
          >
            审核通过
          </Button>
        )}
      </Space>

      <Divider>订单明细</Divider>
      <Table
        columns={itemColumns}
        dataSource={orderItems.map((item, i) => ({ ...item, key: String(i) }))}
        rowKey="key"
        pagination={false}
        locale={{ emptyText: '暂无明细' }}
        scroll={{ x: 700 }}
      />

      {inboundRefs.length > 0 && (
        <>
          <Divider>关联记录</Divider>
          <div style={{ marginBottom: 8 }}>
            <strong>入库单：</strong>
            <Space>
              {inboundRefs.map((ref: InboundRef) => (
                <Button
                  key={ref.inbound_id}
                  type="link"
                  onClick={() => navigate(`/inventory/inbound/${ref.inbound_id}`)}
                >
                  {ref.inbound_no}
                </Button>
              ))}
            </Space>
          </div>
        </>
      )}

      <div style={{ marginTop: 24 }}>
        <Button onClick={() => navigate('/purchases')}>返回列表</Button>
      </div>
    </Card>
  );
}
