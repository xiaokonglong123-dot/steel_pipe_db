<template>
  <nav class="sidebar">
    <div class="sidebar-header">
      <div class="logo">
        <svg width="28" height="28" viewBox="0 0 28 28" fill="none">
          <rect x="2" y="8" width="24" height="12" rx="3" fill="#0071e3"/>
          <rect x="6" y="4" width="4" height="20" rx="2" fill="#0071e3" opacity="0.6"/>
          <rect x="18" y="4" width="4" height="20" rx="2" fill="#0071e3" opacity="0.6"/>
        </svg>
        <span>钢管管理</span>
      </div>
    </div>
    <div class="nav-items">
      <router-link v-for="item in navItems" :key="item.path" :to="item.path" class="nav-item" :class="{ active: $route.path === item.path }">
        <span class="nav-icon">{{ item.icon }}</span>
        <span class="nav-label">{{ item.label }}</span>
      </router-link>
    </div>
  </nav>
</template>

<script setup>
const navItems = [
  { path: '/', label: '首页概览', icon: '◉' },
  { path: '/entry', label: '钢管入库', icon: '↓' },
  { path: '/exit', label: '钢管出库', icon: '↑' },
  { path: '/inventory', label: '库存查询', icon: '⊞' },
  { path: '/records', label: '出入库记录', icon: '≡' },
  { path: '/statistics', label: '数据统计', icon: '◐' },
]
</script>

<style scoped>
.sidebar {
  position: fixed;
  left: 0;
  top: 0;
  bottom: 0;
  width: 240px;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-right: 1px solid var(--apple-border);
  padding: 24px 0;
  z-index: 100;
}

.sidebar-header {
  padding: 0 20px 24px;
  border-bottom: 1px solid var(--apple-border);
  margin-bottom: 16px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 18px;
  font-weight: 600;
  color: var(--apple-text);
  letter-spacing: -0.02em;
}

.nav-items {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 0 12px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border-radius: var(--apple-radius-sm);
  font-size: 14px;
  font-weight: 500;
  color: var(--apple-text-secondary);
  transition: var(--apple-transition);
}

.nav-item:hover {
  background: var(--apple-gray);
  color: var(--apple-text);
}

.nav-item.active {
  background: var(--apple-blue);
  color: white;
}

.nav-icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
}
</style>
