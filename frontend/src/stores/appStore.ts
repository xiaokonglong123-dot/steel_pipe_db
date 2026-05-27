/**
 * 应用全局状态管理 — 侧边栏折叠状态、主题模式（light/dark）
 *
 * 所有状态同时写入 localStorage，页面刷新后自动恢复。
 * toggleSidebar 切换侧边栏折叠，setTheme 切换明暗主题。
 */
import { create } from 'zustand';

type ThemeMode = 'light' | 'dark';

interface AppState {
  sidebarCollapsed: boolean;
  theme: ThemeMode;
  toggleSidebar: () => void;
  setTheme: (theme: ThemeMode) => void;
}

const LS_COLLAPSED = 'sidebar_collapsed';
const LS_THEME = 'app_theme';

export const useAppStore = create<AppState>((set) => ({
  sidebarCollapsed: localStorage.getItem(LS_COLLAPSED) === 'true',
  theme: (localStorage.getItem(LS_THEME) as ThemeMode) || 'light',

  toggleSidebar: () => {
    set((state) => {
      const next = !state.sidebarCollapsed;
      localStorage.setItem(LS_COLLAPSED, String(next));
      return { sidebarCollapsed: next };
    });
  },

  setTheme: (theme) => {
    localStorage.setItem(LS_THEME, theme);
    set({ theme });
  },
}));
