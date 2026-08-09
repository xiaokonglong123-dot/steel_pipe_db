import { useState } from 'react';
import { Modal, Table, Input, Tag, Space, Button } from 'antd';
import { SearchOutlined, InboxOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import apiClient from '@/api/client';
import type { PaginatedResponse } from '@/types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { z } from 'zod';

/** Minimal item shape needed by order forms (matches GET /api/v1/items). */
export interface ItemOption {
  id: number;
  sku: string;
  name: string;
  category?: string | null;
  unit?: string | null;
  spec?: string | null;
  price?: number | null;
  status: string;
}

const itemSchema = z.object({
  id: z.number(),
  sku: z.string(),
  name: z.string(),
  category: z.string().optional(),
  unit: z.string().optional(),
  spec: z.string().optional(),
  price: z.number().optional(),
  status: z.string(),
});

interface ItemPickerProps {
  open: boolean;
  onCancel: () => void;
  /** Called with the selected item master rows when the user confirms. */
  onSelect: (items: ItemOption[]) => void;
  multiple?: boolean;
}

/**
 * Item (商品) picker modal — fetches the item master (`GET /api/v1/items`),
 * supports search by SKU/name and multi-select. Used by order forms to pick
 * line items; replaces the legacy pipe-type pickers.
 */
export default function ItemPicker({
  open,
  onCancel,
  onSelect,
  multiple = true,
}: ItemPickerProps) {
  const { t } = useTranslation();
  const [page, setPage] = useState(1);
  const [q, setQ] = useState('');
  const [data, setData] = useState<{ items: ItemOption[]; total: number }>({ items: [], total: 0 });
  const [loading, setLoading] = useState(false);
  const [selectedIds, setSelectedIds] = useState<number[]>([]);

  const fetchItems = async (p = page, search = q) => {
    setLoading(true);
    try {
      const res = await apiClient.get<PaginatedResponse<ItemOption>>('/items', {
        page: p,
        page_size: 20,
        q: search || undefined,
      } as Record<string, unknown>);
      const parsed = validateResponse(
        paginatedDataSchema(itemSchema),
        res.data,
      );
      setData({ items: parsed.items, total: parsed.total });
      setPage(p);
    } catch {
      // Keep the previous list on failure; the table will show stale data.
    } finally {
      setLoading(false);
    }
  };

  const openModal = () => {
    setQ('');
    setSelectedIds([]);
    fetchItems(1, '');
  };

  const confirm = () => {
    const picked = data.items.filter((it) => selectedIds.includes(it.id));
    onSelect(picked);
    setSelectedIds([]);
  };

  return (
    <Modal
      title={t('items.select_item', '选择商品')}
      open={open}
      onCancel={onCancel}
      afterOpenChange={(visible) => {
        if (visible) openModal();
      }}
      onOk={confirm}
      okText={t('common.confirm', '确定')}
      cancelText={t('common.cancel', '取消')}
      width={720}
      footer={
        multiple
          ? [
              <Button key="cancel" onClick={onCancel}>
                {t('common.cancel', '取消')}
              </Button>,
              <Button key="ok" type="primary" onClick={confirm} disabled={selectedIds.length === 0}>
                {t('common.confirm', '确定')} ({selectedIds.length})
              </Button>,
            ]
          : null
      }
    >
      <Space style={{ marginBottom: 12 }}>
        <Input
          placeholder={t('items.search_placeholder', '搜索 SKU / 名称')}
          prefix={<SearchOutlined />}
          allowClear
          style={{ width: 260 }}
          value={q}
          onChange={(e) => setQ(e.target.value)}
          onPressEnter={() => fetchItems(1, q)}
        />
        <Button type="primary" icon={<SearchOutlined />} onClick={() => fetchItems(1, q)}>
          {t('common.search', '搜索')}
        </Button>
      </Space>
      <Table<ItemOption>
        rowKey="id"
        size="small"
        loading={loading}
        dataSource={data.items}
        rowSelection={
          multiple
            ? {
                selectedRowKeys: selectedIds,
                onChange: (keys) => setSelectedIds(keys as number[]),
              }
            : undefined
        }
        onRow={
          multiple
            ? undefined
            : (record) => ({
                onClick: () => {
                  onSelect([record]);
                  onCancel();
                },
              })
        }
        pagination={{
          current: page,
          pageSize: 20,
          total: data.total,
          onChange: (p) => fetchItems(p, q),
        }}
        columns={[
          { title: t('items.sku', 'SKU'), dataIndex: 'sku', key: 'sku', width: 160 },
          { title: t('items.name', '名称'), dataIndex: 'name', key: 'name' },
          {
            title: t('items.category', '分类'),
            dataIndex: 'category',
            key: 'category',
            width: 120,
            render: (v?: string | null) => v || '-',
          },
          {
            title: t('items.unit', '单位'),
            dataIndex: 'unit',
            key: 'unit',
            width: 80,
            render: (v?: string | null) => v || '-',
          },
          {
            title: t('items.spec', '规格'),
            dataIndex: 'spec',
            key: 'spec',
            ellipsis: true,
            render: (v?: string | null) => v || '-',
          },
          {
            title: t('items.status', '状态'),
            dataIndex: 'status',
            key: 'status',
            width: 100,
            render: (v: string) =>
              v === 'active' ? (
                <Tag color="green">{t('items.status_active', '启用')}</Tag>
              ) : (
                <Tag>{t('items.status_inactive', '停用')}</Tag>
              ),
          },
        ]}
      />
      <div style={{ marginTop: 8, color: '#888', display: 'flex', alignItems: 'center', gap: 6 }}>
        <InboxOutlined /> {t('items.total_items_hint', '共 {{total}} 件商品', { total: data.total })}
      </div>
    </Modal>
  );
}
