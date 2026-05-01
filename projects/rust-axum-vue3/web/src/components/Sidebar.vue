<template>
  <nav class="sidebar">
    <div class="sidebar-header">
      <div class="logo">
        <svg width="32" height="32" viewBox="0 0 32 32" fill="none">
          <rect x="2" y="10" width="28" height="12" rx="3" fill="url(#grad1)"/>
          <rect x="6" y="6" width="4" height="20" rx="2" fill="#0071e3" opacity="0.7"/>
          <rect x="22" y="6" width="4" height="20" rx="2" fill="#0071e3" opacity="0.7"/>
          <defs>
            <linearGradient id="grad1" x1="2" y1="10" x2="30" y2="22" gradientUnits="userSpaceOnUse">
              <stop stop-color="#0071e3"/>
              <stop offset="1" stop-color="#5856d6"/>
            </linearGradient>
          </defs>
        </svg>
        <div class="logo-text">
          <span>钢管管理</span>
          <small>库存系统</small>
        </div>
      </div>
    </div>
    <div class="nav-items">
      <router-link v-for="item in navItems" :key="item.path" :to="item.path" class="nav-item" :class="{ active: $route.path === item.path }">
        <span class="nav-icon" v-html="item.icon"></span>
        <span class="nav-label">{{ item.label }}</span>
      </router-link>
    </div>
    <div class="sidebar-footer">
      <div class="version">v1.0.0</div>
    </div>
  </nav>
</template>

<script setup>
const navItems = [
  { path: '/', label: '仪表盘', icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="9"/><rect x="14" y="3" width="7" height="5"/><rect x="14" y="12" width="7" height="9"/><rect x="3" y="16" width="7" height="5"/></svg>' },
  { path: '/production', label: '生产投料', icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>' },
  { path: '/entry', label: '钢管入库', icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="19" x2="12" y2="5"/><polyline points="5,12 12,5 19,12"/></svg>' },
  { path: '/exit', label: '钢管出库', icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><polyline points="19,12 12,19 5,12"/></svg>' },
  { path: '/inventory', label: '库存查询', icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27,6.96 12,12.01 20.73,6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>' },
  { path: '/records', label: '出入库记录', icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14,2 14,8 20,8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>' },
  { path: '/statistics', label: '数据统计', icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg>' },
  { path: '/heat-treatment', label: '热处理管理', icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"/></svg>' },
]
</script>

<style scoped>
.sidebar {
  position: fixed;
  left: 0;
  top: 0;
  bottom: 0;
  width: 260px;
  background: var(--apple-card);
  border-right: 1px solid var(--apple-border);
  padding: 24px 0;
  z-index: 100;
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  padding: 0 20px 24px;
  border-bottom: 1px solid var(--apple-border);
  margin-bottom: 16px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-text {
  display: flex;
  flex-direction: column;
}

.logo-text span {
  font-size: 17px;
  font-weight: 700;
  color: var(--apple-text);
  letter-spacing: -0.02em;
}

.logo-text small {
  font-size: 11px;
  color: var(--apple-text-secondary);
  font-weight: 500;
}

.nav-items {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 0 12px;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 500;
  color: var(--apple-text-secondary);
  transition: var(--apple-transition);
  text-decoration: none;
}

.nav-item:hover {
  background: var(--apple-gray);
  color: var(--apple-text);
}

.nav-item.active {
  background: linear-gradient(135deg, #0071e3 0%, #5856d6 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(0, 113, 227, 0.3);
}

.nav-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
}

.nav-icon :deep(svg) {
  width: 20px;
  height: 20px;
}

.nav-label {
  flex: 1;
}

.sidebar-footer {
  padding: 16px 20px;
  border-top: 1px solid var(--apple-border);
}

.version {
  font-size: 12px;
  color: var(--apple-text-secondary);
}

@media (max-width: 1024px) {
  .sidebar {
    width: 80px;
    padding: 16px 0;
  }
  
  .sidebar-header {
    padding: 0 16px 16px;
  }
  
  .logo-text {
    display: none;
  }
  
  .nav-label {
    display: none;
  }
  
  .nav-item {
    justify-content: center;
    padding: 12px;
  }
  
  .nav-icon {
    margin: 0;
  }
}
</style>
