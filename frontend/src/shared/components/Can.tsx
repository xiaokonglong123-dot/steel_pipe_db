import React from 'react';
import { useAuthStore } from '@/stores/authStore';

export interface CanProps {
  /** Permission key(s) required to render children; ALL must be present if multiple */
  permission?: string | string[];
  /** Fallback when the user lacks permission (defaults to null) */
  fallback?: React.ReactNode;
  children: React.ReactNode;
}

/**
 * RBAC render gate — shows children only when the current user holds the
 * given permission key(s). Permission keys come from the JWT claims
 * (populated into authStore.user.permissions at login/refresh).
 *
 * Usage:
 * ```tsx
 * <Can permission="pipe.write">
 *   <CreateButton />
 * </Can>
 * ```
 */
export function Can({ permission, fallback = null, children }: CanProps) {
  const user = useAuthStore((s) => s.user);
  const permissions = user?.permissions ?? [];

  if (permission === undefined) return <>{children}</>;

  const required = Array.isArray(permission) ? permission : [permission];
  const allowed = required.every((key) => permissions.includes(key));

  return allowed ? <>{children}</> : <>{fallback}</>;
}
