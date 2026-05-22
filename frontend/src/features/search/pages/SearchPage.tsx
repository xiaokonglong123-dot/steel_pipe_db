import { useState, useCallback } from 'react';
import { Input, Tabs, Table, Empty, Card, Tag } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { useDebounce } from '@/shared/hooks/useDebounce';

import { useSearchPipes, useSearchInbound, useSearchOutbound, useSearchPurchaseOrders, useSearchSalesOrders } from '../api';


const pipeColumns = [
  { title: '编号', dataIndex: 'pipe_number', key: 'pipe_number', width: 180 },
  { title: '钢级', dataIndex: 'grade', key: 'grade', width: 100 },
  { title: '外径', dataIndex: 'od', key: 'od', width: 80, render: (v: number) => `${v}mm` },
  { title: '壁厚', dataIndex: 'wt', key: 'wt', width: 80, render: (v: number) => `${v}mm` },
  { title: '状态', dataIndex: 'status', key: 'status', width: 100, render: (v: string) => <Tag>{v}</Tag> },
];

const inboundColumns = [
  { title: '入库单号', dataIndex: 'inbound_no', key: 'inbound_no', width: 180 },
  { title: '类型', dataIndex: 'inbound_type', key: 'inbound_type', width: 120 },
  { title: '审批状态', dataIndex: 'approval_status', key: 'approval_status', width: 120 },
  { title: '日期', dataIndex: 'created_at', key: 'created_at', width: 180 },
];

const outboundColumns = [
  { title: '出库单号', dataIndex: 'outbound_no', key: 'outbound_no', width: 180 },
  { title: '类型', dataIndex: 'outbound_type', key: 'outbound_type', width: 120 },
  { title: '审批状态', dataIndex: 'approval_status', key: 'approval_status', width: 120 },
  { title: '日期', dataIndex: 'created_at', key: 'created_at', width: 180 },
];

const purchaseColumns = [
  { title: '采购单号', dataIndex: 'order_number', key: 'order_number', width: 180 },
  { title: '供应商', dataIndex: 'supplier_name', key: 'supplier_name', width: 150 },
  { title: '状态', dataIndex: 'status', key: 'status', width: 100 },
  { title: '日期', dataIndex: 'order_date', key: 'order_date', width: 180 },
];

const salesColumns = [
  { title: '销售单号', dataIndex: 'order_number', key: 'order_number', width: 180 },
  { title: '客户', dataIndex: 'customer_name', key: 'customer_name', width: 150 },
  { title: '状态', dataIndex: 'status', key: 'status', width: 100 },
  { title: '日期', dataIndex: 'order_date', key: 'order_date', width: 180 },
];

export default function SearchPage() {
  const { t } = useTranslation('search');
  const [query, setQuery] = useState('');
  const debouncedQuery = useDebounce(query, 300);

  const { data: pipes, isFetching: pipesLoading } = useSearchPipes(debouncedQuery);
  const { data: inbound, isFetching: inboundLoading } = useSearchInbound(debouncedQuery);
  const { data: outbound, isFetching: outboundLoading } = useSearchOutbound(debouncedQuery);
  const { data: purchases, isFetching: purchasesLoading } = useSearchPurchaseOrders(debouncedQuery);
  const { data: sales, isFetching: salesLoading } = useSearchSalesOrders(debouncedQuery);

  const handleSearch = useCallback((value: string) => {
    setQuery(value.trim());
  }, []);

  const tabItems = [
    {
      key: 'pipes',
      label: `${t('tabs.pipes')} (${pipes?.length ?? 0})`,
      children: (
        <Table
          dataSource={pipes}
          columns={pipeColumns}
          rowKey="id"
          loading={pipesLoading}
          locale={{ emptyText: <Empty description={t('empty')} /> }}
          pagination={false}
          size="small"
        />
      ),
    },
    {
      key: 'inbound',
      label: `${t('tabs.inbound')} (${inbound?.length ?? 0})`,
      children: (
        <Table
          dataSource={inbound}
          columns={inboundColumns}
          rowKey="id"
          loading={inboundLoading}
          locale={{ emptyText: <Empty description={t('empty')} /> }}
          pagination={false}
          size="small"
        />
      ),
    },
    {
      key: 'outbound',
      label: `${t('tabs.outbound')} (${outbound?.length ?? 0})`,
      children: (
        <Table
          dataSource={outbound}
          columns={outboundColumns}
          rowKey="id"
          loading={outboundLoading}
          locale={{ emptyText: <Empty description={t('empty')} /> }}
          pagination={false}
          size="small"
        />
      ),
    },
    {
      key: 'purchases',
      label: `${t('tabs.purchases')} (${purchases?.length ?? 0})`,
      children: (
        <Table
          dataSource={purchases}
          columns={purchaseColumns}
          rowKey="id"
          loading={purchasesLoading}
          locale={{ emptyText: <Empty description={t('empty')} /> }}
          pagination={false}
          size="small"
        />
      ),
    },
    {
      key: 'sales',
      label: `${t('tabs.sales')} (${sales?.length ?? 0})`,
      children: (
        <Table
          dataSource={sales}
          columns={salesColumns}
          rowKey="id"
          loading={salesLoading}
          locale={{ emptyText: <Empty description={t('empty')} /> }}
          pagination={false}
          size="small"
        />
      ),
    },
  ];

  return (
    <div>
      <Card style={{ marginBottom: 16 }}>
        <Input.Search
          placeholder={t('placeholder')}
          allowClear
          enterButton={<><SearchOutlined /> {t('search')}</>}
          size="large"
          onSearch={handleSearch}
          onChange={(e) => {
            if (!e.target.value) setQuery('');
          }}
          style={{ maxWidth: 600 }}
        />
      </Card>

      {debouncedQuery ? (
        <Card>
          <Tabs items={tabItems} />
        </Card>
      ) : (
        <Card>
          <Empty description={t('noQuery')} />
        </Card>
      )}
    </div>
  );
}
