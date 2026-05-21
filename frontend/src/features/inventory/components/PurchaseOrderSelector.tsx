import { useState } from 'react';
import { Modal, Table, Input, Tag, Space } from 'antd';
import { SearchOutlined, InboxOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { usePurchases } from '../../purchases/hooks/usePurchases';
import type { PurchaseOrder } from '../../purchases/types';

interface PurchaseOrderSelectorProps {
  open: boolean;
  onCancel: () => void;
  onSelect: (po: PurchaseOrder) => void;
}

const STATUS_COLOR_MAP: Record<string, string> = {
  draft: 'default',
  pending: 'orange',
  approved: 'blue',
  shipped: 'cyan',
  completed: 'green',
  cancelled: 'red',
};

export default function PurchaseOrderSelector({
  open,
  onCancel,
  onSelect,
}: PurchaseOrderSelectorProps) {
  const { t } = useTranslation();
  const [poPage, setPoPage] = useState(1);
  const [poSearch, setPoSearch] = useState('');
  const [selectedId, setSelectedId] = useState<number | null>(null);

  const { data: poData, isLoading } = usePurchases({
    page: poPage,
    page_size: 10,
    q: poSearch || undefined,
  });

  const columns = [
    {
      title: t('purchases.order_number'),
      dataIndex: 'order_number',
      key: 'order_number',
    },
    {
      title: t('purchases.supplier'),
      dataIndex: 'supplier_name',
      key: 'supplier_name',
    },
    {
      title: t('purchases.order_date'),
      dataIndex: 'order_date',
      key: 'order_date',
      render: (val: string) => val?.split('T')[0] ?? '-',
    },
    {
      title: t('purchases.total_amount'),
      dataIndex: 'total_amount',
      key: 'total_amount',
      render: (val: number) => val?.toFixed(2) ?? '-',
    },
    {
      title: t('purchases.status'),
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => (
        <Tag color={STATUS_COLOR_MAP[status] ?? 'default'}>{status}</Tag>
      ),
    },
    {
      title: t('purchases.items'),
      dataIndex: 'items',
      key: 'items',
      render: (items: { length: number }) =>
        items ? `${items.length}` : '-',
    },
  ];

  const handleSelect = () => {
    if (!selectedId || !poData?.items) return;
    const po = poData.items.find((i: PurchaseOrder) => i.id === selectedId);
    if (po) {
      onSelect(po);
    }
  };

  return (
    <Modal
      title={
        <Space>
          <InboxOutlined />
          {t('inbound.select_purchase_order')}
        </Space>
      }
      open={open}
      onCancel={onCancel}
      onOk={handleSelect}
      okText={t('common.confirm')}
      cancelText={t('common.cancel')}
      okButtonProps={{ disabled: !selectedId }}
      width={800}
      destroyOnClose
    >
      <Input
        placeholder={t('common.search')}
        prefix={<SearchOutlined />}
        value={poSearch}
        onChange={(e) => {
          setPoSearch(e.target.value);
          setPoPage(1);
          setSelectedId(null);
        }}
        style={{ marginBottom: 16 }}
        allowClear
      />
      <Table
        columns={columns}
        dataSource={poData?.items}
        rowKey="id"
        loading={isLoading}
        size="small"
        pagination={{
          current: poPage,
          pageSize: 10,
          total: poData?.total,
          onChange: (p) => {
            setPoPage(p);
            setSelectedId(null);
          },
          showSizeChanger: false,
        }}
        rowSelection={{
          type: 'radio',
          selectedRowKeys: selectedId ? [selectedId] : [],
          onChange: (keys) => {
            setSelectedId(keys[0] as number);
          },
        }}
      />
    </Modal>
  );
}
