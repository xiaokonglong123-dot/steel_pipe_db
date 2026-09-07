import { fileURLToPath, URL } from "node:url"
import { defineConfig } from "vite"
import vue from "@vitejs/plugin-vue"
import Components from "unplugin-vue-components/vite"
import { ElementPlusResolver } from "unplugin-vue-components/resolvers"

export default defineConfig({
  plugins: [
    vue(),
    Components({
      // dts:false: 不生成严格 GlobalComponents 声明，避免 el-* 模板组件
      // 类型由宽松退回精确 PropType 而暴露 138 处合法绑定（el-option value/el-tag type）
      // 运行时仍按需分包。
      resolvers: [ElementPlusResolver()],
      dts: false,
    }),
  ],
  resolve: { alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) } },
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:3000",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ""),
      },
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ["vue", "vue-router", "pinia", "@tanstack/vue-query"],
          echarts: ["echarts"],
        },
      },
    },
  },
})