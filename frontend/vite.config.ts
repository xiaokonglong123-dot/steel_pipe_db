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
    chunkSizeWarningLimit: 600,
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (id.includes('node_modules')) {
            if (id.includes('@ant-design/icons')) return 'vendor-icons';
            if (id.includes('antd/es/locale')) return 'vendor-antd-locale';
            if (id.includes('antd/es/date-picker') || id.includes('antd/es/calendar') || id.includes('antd/es/time-picker'))
              return 'vendor-antd-date';
            if (id.includes('antd/es/table')) return 'vendor-antd-table';
            if (id.includes('antd/es/form') || id.includes('antd/es/input') || id.includes('antd/es/select') || id.includes('antd/es/upload') || id.includes('antd/es/color-picker') || id.includes('antd/es/cascader') || id.includes('antd/es/checkbox') || id.includes('antd/es/radio') || id.includes('antd/es/switch') || id.includes('antd/es/rate') || id.includes('antd/es/slider'))
              return 'vendor-antd-input';
            if (id.includes('antd/es/modal') || id.includes('antd/es/drawer') || id.includes('antd/es/popover') || id.includes('antd/es/tooltip') || id.includes('antd/es/popconfirm') || id.includes('antd/es/message') || id.includes('antd/es/notification') || id.includes('antd/es/dropdown'))
              return 'vendor-antd-overlay';
            if (id.includes('antd/es/steps') || id.includes('antd/es/splitter') || id.includes('antd/es/anchor') || id.includes('antd/es/progress') || id.includes('antd/es/badge') || id.includes('antd/es/avatar') || id.includes('antd/es/card') || id.includes('antd/es/carousel') || id.includes('antd/es/collapse') || id.includes('antd/es/descriptions') || id.includes('antd/es/empty') || id.includes('antd/es/image') || id.includes('antd/es/list') || id.includes('antd/es/result') || id.includes('antd/es/segmented') || id.includes('antd/es/skeleton') || id.includes('antd/es/space') || id.includes('antd/es/statistic') || id.includes('antd/es/tabs') || id.includes('antd/es/tag') || id.includes('antd/es/timeline') || id.includes('antd/es/tour') || id.includes('antd/es/tree') || id.includes('antd/es/tree-select') || id.includes('antd/es/watermark'))
              return 'vendor-antd-misc';
            if (id.includes('antd')) return 'vendor-antd';
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
