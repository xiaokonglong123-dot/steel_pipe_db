import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { Select, Spin } from 'antd';
import { supplierApi } from '../api/supplierApi';

interface Props {
  value?: string;
  onChange?: (value: string) => void;
  disabled?: boolean;
  placeholder?: string;
}

export default function SupplierSelect({ value, onChange, disabled, placeholder }: Props) {
  const { data, isLoading } = useQuery({
    queryKey: ['suppliers', { page: 1, page_size: 999 }],
    queryFn: () => supplierApi.list({ page: 1, page_size: 999 }),
  });

  const options =
    data?.data?.data?.map((s) => ({
      label: s.name,
      value: s.id,
    })) || [];

  if (isLoading) return <Spin size="small" />;

  return (
    <Select
      showSearch
      placeholder={placeholder || '请选择供应商'}
      value={value}
      onChange={onChange}
      disabled={disabled}
      options={options}
      filterOption={(input, option) =>
        (option?.label as string ?? '').toLowerCase().includes(input.toLowerCase())
      }
      style={{ width: '100%' }}
    />
  );
}
