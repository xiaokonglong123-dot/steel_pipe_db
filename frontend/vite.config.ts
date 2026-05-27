/**
 * Vite configuration for Steel Pipe DB frontend.
 *
 * Key decisions:
 * - Path alias: `@` → `src/` for clean imports
 * - Dev proxy: `/api/*` → `http://localhost:3000` (backend)
 * - Manual chunk splitting: vendor-ui (React, Ant Design, TanStack, etc.) and
 *   vendor-utils (Axios, Zod) to avoid circular chunk warnings and improve caching.
 * - chunkSizeWarningLimit: 1600 KB — Ant Design is large; this suppresses the
 *   default 500 KB warning without hiding real issues.
 */
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      // Path alias: import from '@/features/...' instead of '../../../features/...'
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    proxy: {
      // Forward all /api requests to the Rust backend in dev mode
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
  build: {
    // Ant Design pushes chunks well above the 500 KB default warning threshold
    chunkSizeWarningLimit: 1600,
    rollupOptions: {
      output: {
        /**
         * Manual chunk splitting strategy:
         * - vendor-ui: Heavy UI libraries (React, Ant Design, TanStack Query, etc.)
         *   — changes rarely, benefits from long-term caching
         * - vendor-utils: Lightweight utility libraries (Axios, Zod)
         *   — separated for parallel loading
         * - All other node_modules: default chunking (shared dependencies)
         * - App code: automatic code-splitting via React.lazy per page
         */
        manualChunks(id: string) {
          if (id.includes('node_modules')) {
            if (
              id.includes('antd') ||
              id.includes('@ant-design/icons') ||
              id.includes('react') ||
              id.includes('scheduler') ||
              id.includes('zustand') ||
              id.includes('@tanstack') ||
              id.includes('dayjs') ||
              id.includes('i18next') ||
              id.includes('react-i18next')
            )
              return 'vendor-ui';
            if (
              id.includes('axios') ||
              id.includes('zod')
            )
              return 'vendor-utils';
          }
        },
      },
    },
  },
});
