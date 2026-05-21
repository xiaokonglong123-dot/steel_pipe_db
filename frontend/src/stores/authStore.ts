// 认证状态管理 — Token 和用户信息与 localStorage 双向同步
// 刷新后状态不丢失，登出时清除全部持久化数据
import { create } from 'zustand';
import type { UserInfo } from '@/types';

interface AuthState {
  token: string | null;
  user: UserInfo | null;
  setAuth: (token: string, user: UserInfo) => void;
  setUser: (user: UserInfo) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  // 初始化时从 localStorage 恢复登录态（刷新保持）
  token: localStorage.getItem('auth_token'),
  user: localStorage.getItem('auth_user')
    ? JSON.parse(localStorage.getItem('auth_user')!)
    : null,
  // 登录成功：写入 store 并持久化到 localStorage
  setAuth: (token, user) => {
    localStorage.setItem('auth_token', token);
    localStorage.setItem('auth_user', JSON.stringify(user));
    set({ token, user });
  },
  setUser: (user) => {
    localStorage.setItem('auth_user', JSON.stringify(user));
    set({ user });
  },
  // 登出：清除内存状态 + 持久化数据
  logout: () => {
    localStorage.removeItem('auth_token');
    localStorage.removeItem('auth_user');
    set({ token: null, user: null });
  },
}));
