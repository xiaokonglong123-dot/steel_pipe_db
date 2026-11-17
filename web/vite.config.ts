import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Vite 8（rolldown）不再接受 `esbuild.jsx` 配置，JSX 由官方 React 插件接管。
export default defineConfig({
  plugins: [react()],
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    sourcemap: false,
    target: 'es2022',
  },
  server: {
    port: 5173,
    proxy: {
      '/api': { target: 'http://127.0.0.1:8080', changeOrigin: false },
    },
  },
})
