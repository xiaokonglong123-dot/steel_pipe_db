import { create } from 'zustand';
import type { UserInfo } from '@/types';

interface AuthState {
  user: UserInfo | null;
  setAuth: (user: UserInfo) => void;
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
  setAuth: (user) => {
    localStorage.setItem('auth_user', JSON.stringify(user));
    set({ user });
  },
  setUser: (user) => {
    localStorage.setItem('auth_user', JSON.stringify(user));
    set({ user });
  },
  logout: () => {
    localStorage.removeItem('auth_user');
    set({ user: null });
  },
}));