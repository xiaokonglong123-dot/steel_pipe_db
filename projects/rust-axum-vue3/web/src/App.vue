<template>
  <div class="app">
    <Sidebar />
    <main class="main-content">
      <header class="page-header">
        <div class="header-info">
          <h1 class="page-title">{{ pageTitle }}</h1>
          <p class="page-subtitle">{{ currentDate }}</p>
        </div>
        <div class="header-actions">
          <button class="btn-icon" @click="refreshData" title="刷新数据">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M23 4v6h-6M1 20v-6h6M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
            </svg>
          </button>
        </div>
      </header>
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" @refresh="refreshData" />
        </transition>
      </router-view>
    </main>
    
    <!-- 全局通知 -->
    <transition name="slide-up">
      <div v-if="notification" class="notification" :class="notification.type">
        {{ notification.message }}
      </div>
    </transition>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import Sidebar from './components/Sidebar.vue'

const route = useRoute()
const lastRefresh = ref(Date.now())
const notification = ref(null)

const showNotification = (message, type = 'error') => {
  notification.value = { message, type }
  setTimeout(() => {
    notification.value = null
  }, 3000)
}

const handleApiError = (event) => {
  showNotification(event.detail, 'error')
}

onMounted(() => {
  window.addEventListener('api-error', handleApiError)
})

onUnmounted(() => {
  window.removeEventListener('api-error', handleApiError)
})

const pageTitle = computed(() => {
  const titles = {
    '/': '仪表盘',
    '/inventory': '库存管理',
    '/entry': '入库操作',
    '/exit': '出库操作',
    '/records': '出入库记录',
    '/statistics': '数据统计',
  }
  return titles[route.path] || '钢管管理系统'
})

const currentDate = computed(() => {
  return new Date().toLocaleDateString('zh-CN', {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  })
})

const refreshData = () => {
  lastRefresh.value = Date.now()
}
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

.app {
  display: flex;
  min-height: 100vh;
  background: var(--apple-bg);
}

.main-content {
  flex: 1;
  margin-left: 260px;
  padding: 32px 40px;
  max-width: calc(100vw - 260px);
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 32px;
  padding-bottom: 24px;
  border-bottom: 1px solid var(--apple-border);
}

.page-title {
  font-size: 36px;
  font-weight: 700;
  letter-spacing: -0.02em;
  margin-bottom: 4px;
  background: linear-gradient(135deg, var(--apple-text) 0%, var(--apple-text-secondary) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

/* 通知样式 */
.notification {
  position: fixed;
  bottom: 40px;
  left: 50%;
  transform: translateX(-50%);
  padding: 12px 24px;
  border-radius: var(--apple-radius);
  background: var(--apple-card);
  box-shadow: var(--apple-shadow-lg);
  color: var(--apple-text);
  z-index: 1000;
  font-weight: 500;
  border: 1px solid var(--apple-border);
}

.notification.error {
  border-color: var(--apple-red);
  color: var(--apple-red);
}

.notification.success {
  border-color: var(--apple-green);
  color: var(--apple-green);
}

.slide-up-enter-active, .slide-up-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-up-enter-from {
  transform: translate(-50%, 20px);
  opacity: 0;
}

.slide-up-leave-to {
  transform: translate(-50%, 20px);
  opacity: 0;
}

.page-subtitle {
  font-size: 15px;
  color: var(--apple-text-secondary);
}

.header-actions {
  display: flex;
  gap: 12px;
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border: none;
  background: var(--apple-card);
  border-radius: 10px;
  cursor: pointer;
  color: var(--apple-text);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  transition: var(--apple-transition);
}

.btn-icon:hover {
  background: var(--apple-gray);
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

@media (max-width: 1024px) {
  .main-content {
    margin-left: 80px;
    padding: 20px;
    max-width: calc(100vw - 80px);
  }
  
  .page-title {
    font-size: 28px;
  }
}
</style>
