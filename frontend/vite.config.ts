/**
 * Vite configuration for Steel Pipe DB frontend.
 *
 * Key decisions:
 * - Path alias: `@` → `src/` for clean imports
 * - Dev proxy: `/api/*` → `http://localhost:3000` (backend)
 * - Manual chunk splitting: vendor-react, vendor-antd, vendor-utils for optimal caching
 * - chunkSizeWarningLimit: 1200 KB — Ant Design is large; this suppresses the
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
    chunkSizeWarningLimit: 1200,
    rollupOptions: {
      output: {
        /**
         * Manual chunk splitting strategy:
         * - vendor-react: React core (changes rarely, benefits from long-term caching)
         * - vendor-antd: Ant Design (large, separate for parallel loading)
         * - vendor-utils: Utility libraries (Zod, dayjs, i18next)
         * - All other node_modules: default chunking
         * - App code: automatic code-splitting via React.lazy per page
         */
        manualChunks(id: string) {
          if (id.includes('node_modules')) {
            // React core
            if (
              id.includes('react') ||
              id.includes('scheduler') ||
              id.includes('react-dom')
            ) {
              return 'vendor-react';
            }
            // Ant Design (separate from React)
            if (
              id.includes('antd') ||
              id.includes('@ant-design')
            ) {
              return 'vendor-antd';
            }
            // Other UI libraries
            if (
              id.includes('zustand') ||
              id.includes('@tanstack')
            ) {
              return 'vendor-ui';
            }
            // Utilities
            if (
              id.includes('zod') ||
              id.includes('dayjs') ||
              id.includes('i18next') ||
              id.includes('react-i18next')
            ) {
              return 'vendor-utils';
            }
          }
        },
      },
    },
  },
});
