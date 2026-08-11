<script setup lang="ts">
import { onErrorCaptured, ref } from "vue"
import { ElMessage } from "element-plus"

const errorCaptured = ref(false)

onErrorCaptured((err) => {
  ElMessage.error(err instanceof Error ? err.message : "未知错误")
  errorCaptured.value = true
  return false
})
</script>

<template>
  <router-view v-slot="{ Component }">
    <transition name="fade" mode="out-in">
      <component :is="Component" />
    </transition>
  </router-view>
</template>
