/**
 * 页面容器 — 标准页面的内容外层 wrapper
 *
 * 提供统一的内边距和布局约束，
 * 所有业务页面的内容放置在此容器内。
 */
import type { ReactNode } from 'react';

interface PageContainerProps {
  children: ReactNode;
}

export default function PageContainer({ children }: PageContainerProps) {
  return <div style={{ padding: 0 }}>{children}</div>;
}
