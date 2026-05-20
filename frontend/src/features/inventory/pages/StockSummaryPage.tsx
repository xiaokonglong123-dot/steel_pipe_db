import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { Card, Statistic, Row, Col, Spin } from 'antd';
import {
  DatabaseOutlined,
  ApartmentOutlined,
  FilterOutlined,
  CheckCircleOutlined,
  ExportOutlined,
  CloseCircleOutlined,
} from '@ant-design/icons';
import { inventoryApi } from '../api/inventoryApi';

export default function StockSummaryPage() {
  const { data, isLoading } = useQuery({
    queryKey: ['stock-summary'],
    queryFn: () => inventoryApi.getStockSummary(),
    refetchInterval: 30000,
  });

  const summary = data?.data?.data;

  const cards = [
    {
      title: '总库存',
      value: summary?.total_in_stock ?? 0,
      icon: <DatabaseOutlined style={{ fontSize: 24, color: '#1B3A5C' }} />,
      color: '#1B3A5C',
    },
    {
      title: '无缝钢管',
      value: summary?.seamless_count ?? 0,
      icon: <ApartmentOutlined style={{ fontSize: 24, color: '#1890ff' }} />,
      color: '#1890ff',
    },
    {
      title: '筛管',
      value: summary?.screen_count ?? 0,
      icon: <FilterOutlined style={{ fontSize: 24, color: '#722ed1' }} />,
      color: '#722ed1',
    },
    {
      title: '在库',
      value: summary?.in_stock_count ?? 0,
      icon: <CheckCircleOutlined style={{ fontSize: 24, color: '#52c41a' }} />,
      color: '#52c41a',
    },
    {
      title: '已出库',
      value: summary?.outbound_count ?? 0,
      icon: <ExportOutlined style={{ fontSize: 24, color: '#1890ff' }} />,
      color: '#1890ff',
    },
    {
      title: '报废',
      value: summary?.scrapped_count ?? 0,
      icon: <CloseCircleOutlined style={{ fontSize: 24, color: '#ff4d4f' }} />,
      color: '#ff4d4f',
    },
  ];

  return (
    <Card title="库存概览" styles={{ body: { padding: 16 } }}>
      <Spin spinning={isLoading}>
        <Row gutter={[16, 16]}>
          {cards.map((c) => (
            <Col xs={12} sm={8} md={6} lg={4} key={c.title}>
              <Card
                hoverable
                styles={{ body: { padding: '20px 16px' } }}
                style={{ borderLeft: `4px solid ${c.color}` }}
              >
                <Statistic
                  title={
                    <span>
                      {c.icon}
                      <span style={{ marginLeft: 8 }}>{c.title}</span>
                    </span>
                  }
                  value={c.value}
                  valueStyle={{ fontSize: 28, fontWeight: 600, color: c.color }}
                />
              </Card>
            </Col>
          ))}
        </Row>
      </Spin>
    </Card>
  );
}
