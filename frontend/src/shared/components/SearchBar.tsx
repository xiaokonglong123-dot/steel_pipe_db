import { Input, Button, Space } from 'antd';
import { SearchOutlined, ReloadOutlined } from '@ant-design/icons';

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  onReset: () => void;
  placeholder?: string;
}

export default function SearchBar({ value, onChange, onReset, placeholder = '搜索...' }: SearchBarProps) {
  return (
    <Space>
      <Input
        placeholder={placeholder}
        prefix={<SearchOutlined />}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        style={{ width: 280 }}
        allowClear
        onPressEnter={() => {}}
      />
      <Button icon={<ReloadOutlined />} onClick={onReset}>重置</Button>
    </Space>
  );
}
