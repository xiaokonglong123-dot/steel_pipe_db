import { useState, useCallback } from 'react';
import type { TablePaginationConfig } from 'antd/es/table';
import { useTranslation } from 'react-i18next';

export function usePagination(defaultPageSize = 20) {
  const { t } = useTranslation('common');
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
    showTotal: (total: number) =>
      t('pagination.total', `共 ${total} 条`).replace('{{total}}', String(total)),
  };

  const reset = useCallback(() => {
    setPage(1);
  }, []);

  return { page, pageSize, pagination, onPaginationChange, reset };
}
