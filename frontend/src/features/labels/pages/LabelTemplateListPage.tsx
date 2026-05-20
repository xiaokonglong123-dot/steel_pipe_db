import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Card, Table, Button, Space, Tag, Popconfirm, message } from 'antd';
import { PlusOutlined, EditOutlined, DeleteOutlined } from '@ant-design/icons';
import type { TablePaginationConfig, ColumnsType } from 'antd/es/table';
import { labelApi, LabelTemplate } from '../api/labelApi';

export default function LabelTemplateListPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [page, setPage] = useState(1);

  const { data, isLoading } = useQuery({
    queryKey: ['label-templates', page],
    queryFn: () => labelApi.listTemplates({ page, page_size: 20 }),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => labelApi.deleteTemplate(id),
    onSuccess: () => {
      message.success('删除成功');
      queryClient.invalidateQueries({ queryKey: ['label-templates'] });
    },
    onError: () => message.error('删除失败'),
  });

  const columns: ColumnsType<LabelTemplate> = [
    { title: '模板名称', dataIndex: 'name', key: 'name', width: 180 },
    { title: '标签尺寸', key: 'size', width: 120, render: (_, r) => `${r.width_mm}×${r.height_mm}mm` },
    { title: '默认', dataIndex: 'is_default', key: 'is_default', width: 80, render: (v: boolean) => v ? <Tag color="blue">默认</Tag> : '-' },
    {
      title: '操作', key: 'action', width: 160, fixed: 'right',
      render: (_, r) => (
        <Space>
          <Button type="link" size="small" icon={<EditOutlined />} onClick={() => navigate(`/labels/templates/${r.id}/edit`)}>编辑</Button>
          <Popconfirm title="确认删除" onConfirm={() => deleteMutation.mutate(r.id)} okText="确定" cancelText="取消">
            <Button type="link" size="small" danger icon={<DeleteOutlined />}>删除</Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <Card title="标签模板管理" styles={{ body: { padding: 16 } }}>
      <Button type="primary" icon={<PlusOutlined />} onClick={() => navigate('/labels/templates/new')} style={{ marginBottom: 16 }}>
        新增模板
      </Button>
      <Table
        columns={columns}
        dataSource={data?.data?.data}
        rowKey="id"
        loading={isLoading}
        locale={{ emptyText: '暂无数据' }}
        scroll={{ x: 640 }}
        pagination={{
          current: data?.data?.meta?.page || 1,
          pageSize: data?.data?.meta?.page_size || 20,
          total: data?.data?.meta?.total || 0,
          showSizeChanger: true,
          onChange: (p) => setPage(p),
        }}
      />
    </Card>
  );
}
