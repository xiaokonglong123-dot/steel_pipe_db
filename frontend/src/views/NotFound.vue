<script setup lang="ts">
import { useRouter } from "vue-router"
const router = useRouter()
function goHome(): void { void router.push("/") }
</script>

<template>
  <section class="not-found">
    <div class="content">
      <div class="code">404</div>
      <h1>页面未找到</h1>
      <p>您访问的页面不存在或已被移除。</p>
      <el-button type="primary" @click="goHome">返回首页</el-button>
    </div>
  </section>
</template>

<style scoped>
.not-found {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 80vh;
}
.content { text-align: center; }
.code {
  font-size: 96px;
  font-weight: bold;
  color: var(--el-color-primary);
  line-height: 1;
  margin-bottom: 16px;
}
h1 { font-size: 20px; margin: 0 0 8px; }
p { color: var(--el-text-color-secondary); margin: 0 0 24px; }
</style>
