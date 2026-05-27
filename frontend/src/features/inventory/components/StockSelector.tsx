import { useState, useMemo } from 'react';
import { Modal, Table, Input, Tag, Space, Badge, Button } from 'antd';
import { SearchOutlined, StockOutlined, PlusOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { useStockQuery } from '../hooks/useInventory';

interface StockSelectorProps {
  open: boolean;
  onCancel: () => void;
  onSelect: (pipes: { pipe_type: string; pipe_id: number }[]) => void;
}

const PIPE_TYPE_COLORS: Record<string, string> = {
  casing: 'blue',
  tubing: 'green',
  coupling: 'purple',
  accessory: 'orange',
  seamless: 'cyan',
  screen: 'magenta',
};

export default function StockSelector({ open, onCancel, onSelect }: StockSelectorProps) {
  const { t } = useTranslation();
  const [stockSearch, setStockSearch] = useState('');
  const [selectedRowKeys, setSelectedRowKeys] = useState<number[]>([]);
  const [typeFilter, setTypeFilter] = useState<string | undefined>();

  const { data: stockData, isLoading } = useStockQuery({
    page: 1,
    page_size: 200,
    pipe_type: typeFilter,
    q: stockSearch || undefined,
  });

  type StockItem = Record<string, unknown> & { id: number; pipe_type: string; status: string };

  const inStockItems = useMemo(() => {
    if (!stockData?.items) return [];
    return stockData.items.filter(
      (item: StockItem) => item.status === 'in_stock',
    );
  }, [stockData]);

  const stockSummary = useMemo(() => {
    const groups: Record<string, number> = {};
    for (const item of inStockItems) {
      groups[item.pipe_type] = (groups[item.pipe_type] || 0) + 1;
    }
    return groups;
  }, [inStockItems]);

  const columns = [
    {
      title: t('stock.pipe_type'),
      dataIndex: 'pipe_type',
      key: 'pipe_type',
      width: 110,
      render: (type: string) => (
        <Tag color={PIPE_TYPE_COLORS[type] ?? 'default'}>
          {t(`pipe_type.${type}`, type)}
        </Tag>
      ),
    },
    {
      title: t('pipes.pipe_number'),
      dataIndex: 'pipe_number',
      key: 'pipe_number',
      width: 160,
    },
    {
      title: t('stock.grade'),
      dataIndex: 'grade',
      key: 'grade',
      width: 100,
    },
    {
      title: 'OD',
      dataIndex: 'od',
      key: 'od',
      width: 80,
      render: (val: number) => val?.toFixed(1),
    },
    {
      title: 'WT',
      dataIndex: 'wt',
      key: 'wt',
      width: 80,
      render: (val: number) => val?.toFixed(1),
    },
    {
      title: t('stock.status'),
      dataIndex: 'status',
      key: 'status',
      width: 100,
      render: (status: string) => (
        <Badge status={status === 'in_stock' ? 'success' : 'default'} text={t(`stock.status.${status}`)} />
      ),
    },
  ];

  const handleSelect = () => {
    const selected = inStockItems
      .filter((item: StockItem) => selectedRowKeys.includes(item.id))
      .map((item: StockItem) => ({ pipe_type: item.pipe_type, pipe_id: item.id }));
    onSelect(selected);
    setSelectedRowKeys([]);
  };

  return (
    <Modal
      title={
        <Space>
          <StockOutlined />
          {t('outbound.select_from_stock')}
        </Space>
      }
      open={open}
      onCancel={() => {
        onCancel();
        setSelectedRowKeys([]);
      }}
      onOk={handleSelect}
      okText={
        <Space>
          <PlusOutlined />
          {t('outbound.add_selected', { count: selectedRowKeys.length })}
        </Space>
      }
      okButtonProps={{ disabled: selectedRowKeys.length === 0 }}
      cancelText={t('common.cancel')}
      width={900}
      destroyOnClose
    >
      <div
        style={{
          marginBottom: 12,
          padding: '8px 12px',
          background: '#f6f8fa',
          borderRadius: 6,
          display: 'flex',
          gap: 12,
          flexWrap: 'wrap',
        }}
      >
        <span style={{ fontWeight: 500, color: '#555' }}>
          {t('outbound.available_stock')}:
        </span>
        {Object.entries(stockSummary).map(([type, count]) => (
          <Tag key={type} color={PIPE_TYPE_COLORS[type] ?? 'default'}>
            {t(`pipe_type.${type}`, type)}: {count}
          </Tag>
        ))}
      </div>

      <Space style={{ marginBottom: 12 }} wrap>
        <Input
          placeholder={t('common.search')}
          prefix={<SearchOutlined />}
          value={stockSearch}
          onChange={(e) => {
            setStockSearch(e.target.value);
            setSelectedRowKeys([]);
          }}
          style={{ width: 200 }}
          allowClear
        />
        <Button
          size="small"
          type={typeFilter === undefined ? 'primary' : 'default'}
          onClick={() => setTypeFilter(undefined)}
        >
          {t('common.all')}
        </Button>
        {Object.keys(stockSummary).map((type) => (
          <Button
            key={type}
            size="small"
            type={typeFilter === type ? 'primary' : 'default'}
            onClick={() => setTypeFilter(typeFilter === type ? undefined : type)}
          >
            {t(`pipe_type.${type}`, type)}
          </Button>
        ))}
      </Space>

      <Table
        columns={columns}
        dataSource={inStockItems}
        rowKey="id"
        loading={isLoading}
        size="small"
        pagination={false}
        scroll={{ y: 360 }}
        rowSelection={{
          selectedRowKeys,
          onChange: (keys) => setSelectedRowKeys(keys as number[]),
        }}
      />
    </Modal>
  );
}
