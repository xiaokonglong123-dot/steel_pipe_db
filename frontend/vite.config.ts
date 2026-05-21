import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (id.includes('node_modules')) {
            if (id.includes('@ant-design') || id.includes('antd')) return 'vendor-antd';
            if (
              id.includes('react') ||
              id.includes('scheduler') ||
              id.includes('zustand') ||
              id.includes('@tanstack') ||
              id.includes('react-i18next')
            )
              return 'vendor-react';
            if (
              id.includes('axios') ||
              id.includes('dayjs') ||
              id.includes('i18next') ||
              id.includes('zod')
            )
              return 'vendor-utils';
          }
        },
      },
    },
  },
});
