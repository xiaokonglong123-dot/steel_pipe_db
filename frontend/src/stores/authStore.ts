/**
 * Auth state management — currently logged-in user info and JWT, backed by localStorage
 *
 * Provides setAuth/setUser to update user, logout to clear auth state.
 * Recovers user from localStorage on page refresh.
 */
import { create } from 'zustand';
import type { UserInfo } from '@/types';

interface AuthState {
  user: UserInfo | null;
  token: string | null;
  setAuth: (user: UserInfo, token: string) => void;
  setUser: (user: UserInfo) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  user: (() => {
    try {
      const raw = localStorage.getItem('auth_user');
      return raw ? JSON.parse(raw) : null;
    } catch {
      localStorage.removeItem('auth_user');
      return null;
    }
  })(),
  token: localStorage.getItem('auth_token'),
  setAuth: (user, token) => {
    localStorage.setItem('auth_user', JSON.stringify(user));
    localStorage.setItem('auth_token', token);
    set({ user, token });
  },
  setUser: (user) => {
    localStorage.setItem('auth_user', JSON.stringify(user));
    set({ user });
  },
  logout: () => {
    localStorage.removeItem('auth_user');
    localStorage.removeItem('auth_token');
    set({ user: null, token: null });
  },
}));