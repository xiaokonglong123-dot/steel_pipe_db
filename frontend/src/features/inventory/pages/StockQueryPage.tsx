// 库存查询页 — 按钢管类型/钢级/库位多维度实时查询库存状态
import { useState } from 'react';
import { Table, Space, Tag, Input, Select } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { useStockQuery, useLocations } from '../hooks/useInventory';
import type { Location } from '../api/inventoryApi';

const PIPE_TYPE_OPTIONS = ['casing', 'tubing', 'coupling', 'accessory'];
const GRADE_OPTIONS = ['H40', 'J55', 'K55', 'N80', 'L80', 'C90', 'T95', 'P110', 'Q125'];

export default function StockQueryPage() {
  const { t } = useTranslation();
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [searchText, setSearchText] = useState('');
  const [gradeFilter, setGradeFilter] = useState<string | undefined>();
  const [pipeTypeFilter, setPipeTypeFilter] = useState<string | undefined>();
  const [locationFilter, setLocationFilter] = useState<number | undefined>();

  const { data, isLoading } = useStockQuery({
    page,
    page_size: pageSize,
    q: searchText || undefined,
    grade: gradeFilter,
    pipe_type: pipeTypeFilter,
    location_id: locationFilter,
  });

  const { data: locations } = useLocations({ active_only: true, page_size: 1000 });

  const columns = [
    {
      title: t('stock.pipe_type'),
      dataIndex: 'pipe_type',
      key: 'pipe_type',
      render: (v: string) => <Tag>{t('pipe_type.' + v)}</Tag>,
    },
    {
      title: t('stock.grade'),
      dataIndex: 'grade',
      key: 'grade',
      render: (v: string) => v ? <Tag color="blue">{v}</Tag> : '-',
    },
    {
      title: t('stock.location'),
      dataIndex: 'full_code',
      key: 'full_code',
      render: (v: string) => v || '-',
    },
    {
      title: t('stock.quantity'),
      dataIndex: 'total_count',
      key: 'total_count',
    },
    {
      title: t('stock.status'),
      dataIndex: 'status',
      key: 'status',
      render: (v: string) => {
        if (!v) return '-';
        const color = v === 'in_stock' ? 'green' : 'red';
        return <Tag color={color}>{t('stock.status.' + v)}</Tag>;
      },
    },
  ];

  return (
    <div>
      <div
        style={{
          display: 'flex',
          marginBottom: 16,
          flexWrap: 'wrap',
          gap: 8,
        }}
      >
        <Space wrap>
          <Input
            placeholder={t('common.search')}
            prefix={<SearchOutlined />}
            value={searchText}
            onChange={(e) => {
              setSearchText(e.target.value);
              setPage(1);
            }}
            style={{ width: 200 }}
          />
          <Select
            placeholder={t('stock.grade')}
            allowClear
            style={{ width: 120 }}
            value={gradeFilter}
            onChange={(v) => {
              setGradeFilter(v);
              setPage(1);
            }}
            options={GRADE_OPTIONS.map((g) => ({ label: g, value: g }))}
          />
          <Select
            placeholder={t('stock.pipe_type')}
            allowClear
            style={{ width: 140 }}
            value={pipeTypeFilter}
            onChange={(v) => {
              setPipeTypeFilter(v);
              setPage(1);
            }}
            options={PIPE_TYPE_OPTIONS.map((pt) => ({ label: t('pipe_type.' + pt), value: pt }))}
          />
          <Select
            placeholder={t('stock.location')}
            allowClear
            style={{ width: 200 }}
            value={locationFilter}
            onChange={(v) => {
              setLocationFilter(v);
              setPage(1);
            }}
            options={(locations?.items ?? []).map((loc: Location) => ({
              label: loc.full_code,
              value: loc.id,
            }))}
          />
        </Space>
      </div>

      <Table
        columns={columns}
        dataSource={data?.items as Record<string, unknown>[]}
        rowKey={(record) => `${record.pipe_type}-${record.grade}-${record.location_id || 0}-${page}`}
        loading={isLoading}
        pagination={{
          current: page,
          pageSize,
          total: data?.total,
          onChange: (p, ps) => {
            setPage(p);
            setPageSize(ps);
          },
          showSizeChanger: true,
          showTotal: (total) => t('common.total_items', { total }),
        }}
      />
    </div>
  );
}
