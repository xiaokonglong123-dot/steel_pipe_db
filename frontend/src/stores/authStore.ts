/**
 * Auth state management — in-memory only (no localStorage)
 *
 * Access token lives in memory; refresh token is in httpOnly cookie (backend-managed).
 * On page refresh, auth is restored via /auth/refresh using the cookie.
 */
import { create } from 'zustand';
import type { UserInfo } from '@/types';

interface AuthState {
  user: UserInfo | null;
  token: string | null;
  /** true while attempting to restore session from refresh cookie */
  isRestoring: boolean;
  setAuth: (user: UserInfo, token: string) => void;
  setUser: (user: UserInfo) => void;
  setToken: (token: string) => void;
  setRestoring: (v: boolean) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  token: null,
  isRestoring: true,
  setAuth: (user, token) => {
    set({ user, token, isRestoring: false });
  },
  setUser: (user) => {
    set({ user });
  },
  setToken: (token) => {
    set({ token });
  },
  setRestoring: (v) => {
    set({ isRestoring: v });
  },
  logout: () => {
    set({ user: null, token: null, isRestoring: false });
  },
}));