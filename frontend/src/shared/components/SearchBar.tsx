/**
 * Search bar — keyword input + reset button
 *
 * Controlled component — the parent owns the search term via value/onChange,
 * and onReset wipes the search back to square one.
 */
import { Input, Button, Space } from 'antd';
import { SearchOutlined, ReloadOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  onReset: () => void;
  placeholder?: string;
}

export default function SearchBar({ value, onChange, onReset, placeholder }: SearchBarProps) {
  const { t } = useTranslation('common');
  return (
    <Space>
      <Input
        placeholder={placeholder ?? t('common.search')}
        prefix={<SearchOutlined />}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        style={{ width: 280 }}
        allowClear
        onPressEnter={() => {}}
      />
      <Button icon={<ReloadOutlined />} onClick={onReset}>{t('common.reset')}</Button>
    </Space>
  );
}
