import React from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import {
  Card,
  Descriptions,
  Table,
  Tag,
  Button,
  Space,
  Spin,
  Typography,
  Empty,
  Divider,
} from 'antd';
import { ArrowLeftOutlined, EditOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { qualityApi, QualityCert, QualityCertItem } from '../api/qualityApi';

const { Title } = Typography;

const resultConfig: Record<string, { color: string; label: string }> = {
  pass: { color: 'green', label: '合格' },
  fail: { color: 'red', label: '不合格' },
  pending: { color: 'orange', label: '待检' },
};

const pipeTypeMap: Record<string, string> = {
  casing: '套管',
  tubing: '油管',
  plain_end: '光管',
};

async function fetchCertById(id: string): Promise<{ cert: QualityCert; pipeType: string }> {
  const types = ['casing', 'tubing', 'plain_end'];
  for (const pt of types) {
    try {
      const res = await qualityApi.getCert(pt, id);
      if (res.data?.data) {
        return { cert: res.data.data, pipeType: pt };
      }
    } catch {
      /* try next type */
    }
  }
  throw new Error('未找到质检证书');
}

function parseItems(itemsJson?: string): QualityCertItem[] {
  if (!itemsJson) return [];
  try {
    const parsed = JSON.parse(itemsJson);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

export default function QualityDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const { data, isLoading, error } = useQuery({
    queryKey: ['quality-cert-detail', id],
    queryFn: () => fetchCertById(id!),
    enabled: Boolean(id),
  });

  if (isLoading) {
    return (
      <Card>
        <div style={{ textAlign: 'center', padding: 40 }}>
          <Spin size="large" />
        </div>
      </Card>
    );
  }

  if (error || !data) {
    return (
      <Card>
        <Empty description="质检证书不存在或加载失败" />
        <div style={{ textAlign: 'center', marginTop: 16 }}>
          <Button onClick={() => navigate('/quality/certs')}>
            返回列表
          </Button>
        </div>
      </Card>
    );
  }

  const { cert } = data;
  const items = parseItems(cert.items_json);

  const itemColumns: ColumnsType<QualityCertItem> = [
    { title: '序号', dataIndex: 'item_no', key: 'item_no', width: 60 },
    { title: '检验项目', dataIndex: 'test_item', key: 'test_item', width: 160 },
    { title: '规格要求', dataIndex: 'specification', key: 'specification', width: 160 },
    { title: '实测值', dataIndex: 'measured_value', key: 'measured_value', width: 120 },
    {
      title: '结果',
      dataIndex: 'result',
      key: 'result',
      width: 80,
      render: (v: string) => {
        const cfg = resultConfig[v] || { color: 'default', label: v };
        return <Tag color={cfg.color}>{cfg.label}</Tag>;
      },
    },
    { title: '备注', dataIndex: 'remark', key: 'remark', width: 120, ellipsis: true },
  ];

  return (
    <Card
      title={
        <Space>
          <Button
            type="text"
            icon={<ArrowLeftOutlined />}
            onClick={() => navigate('/quality/certs')}
          />
          <span>质检证书详情</span>
        </Space>
      }
      extra={
        <Button
          type="primary"
          icon={<EditOutlined />}
          onClick={() => navigate(`/quality/certs/${cert.id}/edit`)}
        >
          编辑
        </Button>
      }
      styles={{ body: { padding: 24 } }}
    >
      <Descriptions
        column={{ xs: 1, sm: 2, md: 3 }}
        bordered
        size="small"
        style={{ marginBottom: 24 }}
      >
        <Descriptions.Item label="证书编号">{cert.cert_no}</Descriptions.Item>
        <Descriptions.Item label="管材类型">
          {pipeTypeMap[cert.pipe_type] || cert.pipe_type}
        </Descriptions.Item>
        <Descriptions.Item label="关联管材ID">{cert.pipe_id}</Descriptions.Item>
        <Descriptions.Item label="检验日期">{cert.inspect_date}</Descriptions.Item>
        <Descriptions.Item label="检验人">{cert.inspector}</Descriptions.Item>
        <Descriptions.Item label="检验机构">{cert.agency || '-'}</Descriptions.Item>
        <Descriptions.Item label="检验结果">
          <Tag color={resultConfig[cert.result]?.color}>
            {resultConfig[cert.result]?.label || cert.result}
          </Tag>
        </Descriptions.Item>
        <Descriptions.Item label="创建时间">{cert.created_at}</Descriptions.Item>
        <Descriptions.Item label="更新时间">{cert.updated_at}</Descriptions.Item>
        <Descriptions.Item label="备注" span={3}>
          {cert.notes || '-'}
        </Descriptions.Item>
      </Descriptions>

      {items.length > 0 && (
        <>
          <Divider />
          <Title level={5}>检验项目明细</Title>
          <Table
            columns={itemColumns}
            dataSource={items}
            rowKey="item_no"
            size="small"
            bordered
            pagination={false}
            locale={{ emptyText: '暂无检验项目' }}
          />
        </>
      )}
    </Card>
  );
}
