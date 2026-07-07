import React from 'react';
import { Table, Empty, Spin } from 'antd';
import type { TableProps, TablePaginationConfig } from 'antd';
import { useTranslation } from 'react-i18next';

export interface DataTableProps<T> extends Omit<TableProps<T>, 'pagination'> {
  items?: T[];
  total?: number;
  page?: number;
  pageSize?: number;
  loading?: boolean;
  emptyText?: string;
  onPaginationChange?: (page: number, pageSize: number) => void;
  selectable?: boolean;
  selectedRowKeys?: React.Key[];
  onSelectionChange?: (selectedRowKeys: React.Key[], selectedRows: T[]) => void;
}

function DataTableInner<T extends object>({
  items = [],
  total = 0,
  page = 1,
  pageSize = 20,
  loading = false,
  emptyText,
  onPaginationChange,
  selectable = false,
  selectedRowKeys,
  onSelectionChange,
  columns,
  rowKey = 'id',
  ...rest
}: DataTableProps<T>) {
  const { t } = useTranslation('common');
  const displayEmpty = emptyText || t('common.no_data', '暂无数据');

  const pagination: TablePaginationConfig = {
    current: page,
    pageSize,
    total,
    showSizeChanger: true,
    showQuickJumper: true,
    showTotal: (total, range) =>
      t('pagination.range', `第 ${range[0]}-${range[1]} 条，共 ${total} 条`)
        .replace('{{from}}', String(range[0]))
        .replace('{{to}}', String(range[1]))
        .replace('{{total}}', String(total)),
    onChange: onPaginationChange,
  };

  const rowSelection = selectable
    ? { selectedRowKeys, onChange: onSelectionChange }
    : undefined;

  return (
    <Table<T>
      columns={columns}
      dataSource={items}
      rowKey={rowKey}
      pagination={pagination}
      rowSelection={rowSelection}
      locale={{ emptyText: <Empty description={displayEmpty} /> }}
      loading={{ spinning: loading, indicator: <Spin size="large" /> }}
      {...rest}
    />
  );
}

export const DataTable = React.memo(DataTableInner) as typeof DataTableInner;

export default DataTable;
