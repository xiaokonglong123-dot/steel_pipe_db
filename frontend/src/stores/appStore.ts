// 应用状态管理 — 侧边栏折叠、主题模式，与 localStorage 双向同步
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
