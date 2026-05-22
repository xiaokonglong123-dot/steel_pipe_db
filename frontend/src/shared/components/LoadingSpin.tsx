import { Spin } from 'antd';

interface LoadingSpinProps {
  tip?: string;
}

export default function LoadingSpin({ tip = '加载中...' }: LoadingSpinProps) {
  return (
    <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', padding: 48 }}>
      <Spin tip={tip} />
    </div>
  );
}
