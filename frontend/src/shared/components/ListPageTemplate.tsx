import React, { useMemo, useCallback } from 'react';
import { Table, TableProps, TablePaginationConfig, Card, Popconfirm } from 'antd';
import { useNavigate } from 'react-router-dom';
import { PageLayout } from './PageLayout';
import { SearchBar } from './SearchBar';
import { ActionButton } from './ActionButton';

export interface ListPageTemplateProps<T> {
  /** Page title (i18n key or translated string) */
  title: string;
  /** Path for create button, if provided shows create button */
  createPath?: string;
  /** Table column definitions */
  columns: TableProps<T>['columns'];
  /** Data array for the table */
  data: T[];
  /** Pagination configuration */
  pagination: TablePaginationConfig;
  /** Search handler */
  onSearch: (keyword: string) => void;
  /** Pagination change handler */
  onPageChange: (page: number, pageSize: number) => void;
  /** Delete handler */
  onDelete: (id: string | number) => void;
  /** Edit handler (optional, for row click or action) */
  onEdit?: (id: string | number) => void;
  /** View handler (optional, for row click or action) */
  onView?: (id: string | number) => void;
  /** Loading state */
  loading?: boolean;
  /** Search placeholder (i18n key) */
  searchPlaceholder?: string;
  /** Row key field name (default: 'id') */
  rowKey?: keyof T;
  /** Custom render for actions column */
  renderActions?: (record: T, index: number) => React.ReactNode;
}

/**
 * Reusable list page template that eliminates boilerplate across all list pages.
 * 
 * Provides:
 * - Consistent page layout with title, breadcrumbs, and actions
 * - Search bar with debounced input
 * - Table with pagination, loading, and row selection
 * - Standardized action buttons (view, edit, delete with confirmation)
 * - Empty state handling
 * 
 * Usage:
 * ```tsx
 * const columns = useMemo<ColumnsType<SeamlessPipe>>(() => [
 *   { title: t('pipes.pipe_number'), dataIndex: 'pipe_number', width: 150 },
 *   { title: t('pipes.grade'), dataIndex: 'grade', render: (v) => <StatusTag status={v} /> },
 *   { title: t('pipes.status'), dataIndex: 'status', render: (v) => <StatusTag status={v} /> },
 *   {
 *     title: t('common.actions'),
 *     render: (_, record) => (
 *       <Space>
 *         <ActionButton onClick={() => navigate(`/pipes/seamless/${record.id}`)}>{t('common.view')}</ActionButton>
 *         <ActionButton onClick={() => navigate(`/pipes/seamless/${record.id}/edit`)}>{t('common.edit')}</ActionButton>
 *         <Popconfirm title={t('common.delete_confirm')} onConfirm={() => deleteMutation.mutate(record.id)}>
 *           <ActionButton danger>{t('common.delete')}</ActionButton>
 *         </Popconfirm>
 *       </Space>
 *     ),
 *   },
 * ], [t, navigate]);
 * 
 * return (
 *   <ListPageTemplate
 *     title={t('pipes.seamless_list')}
 *     createPath="/pipes/seamless/new"
 *     columns={columns}
 *     data={data?.items ?? []}
 *     pagination={{ current: page, pageSize, total: data?.meta.total ?? 0 }}
 *     onSearch={setSearchKeyword}
 *     onPageChange={onPaginationChange}
 *     onDelete={(id) => deleteMutation.mutate(id)}
 *     loading={isLoading}
 *   />
 * );
 * ```
 */
export function ListPageTemplate<T>({
  title,
  createPath,
  columns,
  data,
  pagination,
  onSearch,
  onPageChange,
  onDelete,
  onEdit,
  onView,
  loading,
  searchPlaceholder,
  rowKey = 'id' as keyof T,
  renderActions,
}: ListPageTemplateProps<T>) {
  const navigate = useNavigate();

  // Default actions column if not provided
  const defaultActions = useMemo(() => {
    if (!renderActions && (onView || onEdit || onDelete)) {
      return {
        title: 'common.actions',
        dataIndex: 'actions',
        key: 'actions',
        width: 200,
        fixed: 'right' as const,
        render: (_: unknown, record: T) => {
          const id = record[rowKey as keyof T] as string | number;
          return (
            <React.Fragment>
              {onView && (
                <ActionButton
                  onClick={() => onView!(id)}
                  style={{ marginRight: 8 }}
                >
                  View
                </ActionButton>
              )}
              {onEdit && (
                <ActionButton
                  onClick={() => onEdit!(id)}
                  style={{ marginRight: 8 }}
                >
                  Edit
                </ActionButton>
              )}
              {onDelete && (
                <Popconfirm
                  title="Are you sure?"
                  onConfirm={() => onDelete(id)}
                  okText="Yes"
                  cancelText="No"
                >
                  <ActionButton danger>Delete</ActionButton>
                </Popconfirm>
              )}
            </React.Fragment>
          );
        },
      };
    }
    return null;
  }, [onView, onEdit, onDelete, renderActions, rowKey]);

  // Merge custom columns with actions column if needed
  const finalColumns = useMemo(() => {
    const cols = columns ?? [];
    if (!renderActions && defaultActions) {
      return [...cols, defaultActions];
    }
    return cols;
  }, [columns, defaultActions, renderActions]);

  const handleTableChange = useCallback(
    (
      pagination: TablePaginationConfig,
      _filters: Record<string, unknown>,
      _sorter: unknown,
      _extra: unknown
    ) => {
      if (pagination.current && pagination.pageSize) {
        onPageChange(pagination.current, pagination.pageSize);
      }
    },
    [onPageChange]
  );

  return (
    <PageLayout
      title={title}
      actions={createPath ? (
        <ActionButton
          onClick={() => { navigate(createPath); }}
        >
          Create
        </ActionButton>
      ) : null}
    >
      <Card>
        <SearchBar
          placeholder={searchPlaceholder ?? 'common.search_placeholder'}
          onSearch={onSearch}
          style={{ marginBottom: 16 }}
        />
        <Table<T>
          columns={finalColumns}
          dataSource={data}
          rowKey={String(rowKey)}
          loading={loading}
          pagination={pagination}
          onChange={handleTableChange}
        />
      </Card>
    </PageLayout>
  );
}