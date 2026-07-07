import React, { useState, useCallback, useEffect } from 'react';
import { Input } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import { useDebounce } from '../hooks/useDebounce';

export interface SearchBarProps {
  /** Search placeholder text (i18n key) */
  placeholder?: string;
  /** Search handler */
  onSearch: (keyword: string) => void;
  /** Debounce delay in ms */
  debounceMs?: number;
  /** Initial value */
  initialValue?: string;
  /** Additional style */
  style?: React.CSSProperties;
  /** Whether to show clear button */
  allowClear?: boolean;
}

export const SearchBar = React.memo(function SearchBar({
  placeholder,
  onSearch,
  debounceMs = 300,
  initialValue = '',
  style,
  allowClear = true,
}: SearchBarProps) {
  const [value, setValue] = useState(initialValue);
  const debouncedValue = useDebounce(value, debounceMs);

  // Call onSearch when debounced value changes
  useEffect(() => {
    onSearch(debouncedValue);
  }, [debouncedValue, onSearch]);

  const handleChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    setValue(e.target.value);
  }, []);

  return (
    <Input
      placeholder={placeholder}
      value={value}
      onChange={handleChange}
      allowClear={allowClear}
      onPressEnter={() => onSearch(value)}
      prefix={<SearchOutlined />}
      style={{ width: '100%', maxWidth: 300, ...style }}
    />
  );
});