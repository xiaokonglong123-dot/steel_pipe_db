/**
 * DataTable — Modern Ant Design Table wrapper with loading, empty, and pagination.
 *
 * Encapsulates common table patterns:
 * - Loading state
 * - Empty state
 * - Pagination
 * - Row selection
 * - Action columns
 */
import { Table, Empty, Spin } from 'antd';
import type { TableProps, TablePaginationConfig } from 'antd';

export interface DataTableProps<T> extends Omit<TableProps<T>, 'pagination'> {
  /** Data items to display */
  items?: T[];
  /** Total count for server-side pagination */
  total?: number;
  /** Current page (1-indexed) */
  page?: number;
  /** Page size */
  pageSize?: number;
  /** Loading state */
  loading?: boolean;
  /** Empty state description */
  emptyText?: string;
  /** Callback when pagination changes */
  onPaginationChange?: (page: number, pageSize: number) => void;
  /** Show row selection checkbox */
  selectable?: boolean;
  /** Selected row keys */
  selectedRowKeys?: React.Key[];
  /** Callback when selection changes */
  onSelectionChange?: (selectedRowKeys: React.Key[], selectedRows: T[]) => void;
}

export function DataTable<T extends object>({
  items = [],
  total = 0,
  page = 1,
  pageSize = 20,
  loading = false,
  emptyText = '暂无数据',
  onPaginationChange,
  selectable = false,
  selectedRowKeys,
  onSelectionChange,
  columns,
  rowKey = 'id',
  ...rest
}: DataTableProps<T>) {
  const pagination: TablePaginationConfig = {
    current: page,
    pageSize,
    total,
    showSizeChanger: true,
    showQuickJumper: true,
    showTotal: (total, range) => `第 ${range[0]}-${range[1]} 条，共 ${total} 条`,
    onChange: onPaginationChange,
  };

  const rowSelection = selectable
    ? {
        selectedRowKeys,
        onChange: onSelectionChange,
      }
    : undefined;

  return (
    <Table<T>
      columns={columns}
      dataSource={items}
      rowKey={rowKey}
      pagination={pagination}
      rowSelection={rowSelection}
      locale={{
        emptyText: <Empty description={emptyText} />,
      }}
      loading={{
        spinning: loading,
        indicator: <Spin size="large" />,
      }}
      {...rest}
    />
  );
}

export default DataTable;
