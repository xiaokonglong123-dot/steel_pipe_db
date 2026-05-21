// 路由守卫 — 未登录跳转登录页，可选角色鉴权
import { Navigate } from 'react-router-dom';
import { useAuthStore } from '@/stores/authStore';
import type { ReactNode } from 'react';

interface ProtectedRouteProps {
  children: ReactNode;
  roles?: string[];
}

export default function ProtectedRoute({ children, roles }: ProtectedRouteProps) {
  const token = useAuthStore((s) => s.token);
  const user = useAuthStore((s) => s.user);

  // 未登录 → 登录页
  if (!token) {
    return <Navigate to="/login" replace />;
  }

  // 已登录但角色不符 → 首页（不展示 403，静默降级）
  if (roles && user && !roles.includes(user.role)) {
    return <Navigate to="/" replace />;
  }

  return <>{children}</>;
}
