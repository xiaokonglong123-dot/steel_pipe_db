<template>
  <div class="app">
    <Sidebar />
    <main class="main-content">
      <router-view />
    </main>
  </div>
</template>

<script setup>
import Sidebar from './components/Sidebar.vue'
</script>

<style>
.app {
  display: flex;
  min-height: 100vh;
}

.main-content {
  margin-left: 240px;
  flex: 1;
  padding: 32px 40px;
  max-width: 1200px;
}

@media (max-width: 768px) {
  .sidebar { display: none; }
  .main-content { margin-left: 0; padding: 20px; }
}
</style>
