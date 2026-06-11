import { useState, useCallback } from 'react';
import type { TablePaginationConfig } from 'antd/es/table';

/** Generic pagination state for list pages */
export function usePagination(defaultPageSize = 20) {
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(defaultPageSize);

  const onPaginationChange = useCallback(
    (newPage: number, newPageSize: number) => {
      setPage(newPage);
      setPageSize(newPageSize);
    },
    [],
  );

  const pagination: TablePaginationConfig = {
    current: page,
    pageSize,
    onChange: onPaginationChange,
    showSizeChanger: true,
    showTotal: (total: number) => `共 ${total} 条`,
  };

  const reset = useCallback(() => {
    setPage(1);
  }, []);

  return { page, pageSize, pagination, onPaginationChange, reset };
}
