import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { Select, Spin } from 'antd';
import { customerApi } from '../api/customerApi';

interface Props {
  value?: string;
  onChange?: (value: string) => void;
  disabled?: boolean;
  placeholder?: string;
}

export default function CustomerSelect({ value, onChange, disabled, placeholder }: Props) {
  const { data, isLoading } = useQuery({
    queryKey: ['customers', { page: 1, page_size: 999 }],
    queryFn: () => customerApi.list({ page: 1, page_size: 999 }),
  });

  const options =
    data?.data?.data?.map((c) => ({
      label: c.name,
      value: c.id,
    })) || [];

  if (isLoading) return <Spin size="small" />;

  return (
    <Select
      showSearch
      placeholder={placeholder || '请选择客户'}
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
