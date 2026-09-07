<script setup lang="ts">
defineProps<{ title: string; subtitle?: string | undefined }>()
</script>

<template>
  <div class="page-heading">
    <div>
      <h2>{{ title }}</h2>
      <p v-if="subtitle">{{ subtitle }}</p>
    </div>
    <div class="page-heading-actions"><slot /></div>
  </div>
</template>

<style scoped>
.page-heading { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 16px; gap: 16px; }
.page-heading h2 { margin: 0 0 6px; }
.page-heading p { margin: 0; color: var(--muted); }
.page-heading-actions { display: flex; gap: 8px; flex-shrink: 0; }
</style>