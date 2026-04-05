<template>
  <div class="dashboard">
    <div class="stats-grid" v-if="stats">
      <div class="stat-card" v-for="(stat, index) in statCards" :key="index" :style="{ '--accent': stat.color }">
        <div class="stat-icon">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path :d="stat.icon"/>
          </svg>
        </div>
        <div class="stat-content">
          <div class="stat-label">{{ stat.label }}</div>
          <div class="stat-value">{{ stat.value }}</div>
          <div class="stat-change" :class="stat.changeClass" v-if="stat.change">
            <span>{{ stat.change }}</span>
          </div>
        </div>
      </div>
    </div>
    <div v-else class="skeleton-grid">
      <div v-for="i in 4" :key="i" class="skeleton-card"></div>
    </div>

    <div class="content-grid">
      <div class="card recent-activity">
        <div class="card-header">
          <h2 class="card-title">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/><polyline points="12,6 12,12 16,14"/>
            </svg>
            最近操作
          </h2>
          <router-link to="/records" class="view-all">查看全部 →</router-link>
        </div>
        <div class="record-list" v-if="recentRecords.length">
          <div v-for="r in recentRecords" :key="r.id" class="record-item">
            <div class="record-icon" :class="r.operation_type === '入库' ? 'in' : 'out'">
              <svg v-if="r.operation_type === '入库'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19"/><polyline points="19,12 12,19 5,12"/>
              </svg>
              <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="19" x2="12" y2="5"/><polyline points="5,12 12,5 19,12"/>
              </svg>
            </div>
            <div class="record-info">
              <span class="pipe-id">{{ r.pipe_id }}</span>
              <span class="qty">{{ r.quantity }} 根</span>
            </div>
            <span class="op-badge" :class="r.operation_type === '入库' ? 'in' : 'out'">{{ r.operation_type }}</span>
            <span class="date">{{ formatDate(r.operation_date) }}</span>
          </div>
        </div>
        <div v-else class="empty">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" opacity="0.3">
            <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="13,2 13,9 20,9"/>
          </svg>
          <span>暂无操作记录</span>
        </div>
      </div>

      <div class="card material-stats">
        <div class="card-header">
          <h2 class="card-title">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21.21 15.89A10 10 0 1 1 8 2.83"/><path d="M22 12A10 10 0 0 0 12 2v10z"/>
            </svg>
            材质分布
          </h2>
        </div>
        <div class="material-list" v-if="materialStats.length">
          <div v-for="(m, i) in materialStats" :key="m.material" class="material-item">
            <div class="material-bar">
              <div class="material-fill" :style="{ width: getPercent(m.total_quantity) + '%', background: barColors[i % barColors.length] }"></div>
            </div>
            <div class="material-info">
              <span class="material-name">{{ m.material }}</span>
              <span class="material-stats-info">{{ m.type_count }} 种 · {{ m.total_quantity }} 根</span>
            </div>
          </div>
        </div>
        <div v-else class="empty">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" opacity="0.3">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="21" x2="9" y2="9"/>
          </svg>
          <span>暂无数据</span>
        </div>
      </div>
    </div>

    <div class="alert-card" v-if="lowStock.length">
      <div class="alert-header">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
        </svg>
        <span>库存预警</span>
        <span class="alert-count">{{ lowStock.length }} 项</span>
      </div>
      <div class="alert-list">
        <div v-for="p in lowStock.slice(0, 5)" :key="p.id" class="alert-item">
          <span class="pipe-id">{{ p.pipe_id }}</span>
          <span class="pipe-spec">{{ p.diameter.toFixed(1) }}mm × {{ p.thickness.toFixed(1) }}mm × {{ p.length.toFixed(1) }}m</span>
          <span class="warning-qty">{{ p.quantity }} 根</span>
        </div>
      </div>
      <router-link to="/inventory?status=低库存" class="alert-action">
        查看全部预警 →
      </router-link>
    </div>

    <div v-if="errorMsg" class="toast error">
      {{ errorMsg }}
      <button @click="errorMsg = ''">×</button>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { statsAPI, recordsAPI } from '../api'

const stats = ref(null)
const recentRecords = ref([])
const materialStats = ref([])
const lowStock = ref([])
const errorMsg = ref('')

const barColors = ['#0071e3', '#34c759', '#5856d6', '#ff9500', '#ff3b30', '#af52de']

const statCards = computed(() => {
  if (!stats.value) return []
  return [
    { label: '钢管种类', value: stats.value.total_types, color: '#0071e3', icon: 'M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4' },
    { label: '库存总量', value: stats.value.total_quantity, color: '#34c759', icon: 'M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z' },
    { label: '入库总数', value: stats.value.total_in, color: '#5856d6', icon: 'M12 19V5M5 12l7-7 7 7', change: '+12%', changeClass: 'positive' },
    { label: '出库总数', value: stats.value.total_out, color: '#ff9500', icon: 'M5 12h14M12 5l7 7-7 7', change: '-5%', changeClass: 'negative' },
  ]
})

const getPercent = (val) => {
  if (!materialStats.value.length) return 0
  const max = Math.max(...materialStats.value.map(m => m.total_quantity))
  return max > 0 ? (val / max * 100) : 0
}

const formatDate = (dateStr) => {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now - date
  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return Math.floor(diff / 60000) + '分钟前'
  if (diff < 86400000) return Math.floor(diff / 3600000) + '小时前'
  return date.toLocaleDateString('zh-CN')
}

onMounted(async () => {
  try {
    const [s, ms, ls, rr] = await Promise.all([
      statsAPI.overview(),
      statsAPI.byMaterial(),
      statsAPI.lowStock(10),
      recordsAPI.list({}),
    ])
    stats.value = s.data
    materialStats.value = ms.data
    lowStock.value = ls.data
    recentRecords.value = rr.data.slice(0, 8)
  } catch (e) {
    errorMsg.value = e.message || '加载数据失败'
  }
})
</script>

<style scoped>
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 20px;
  margin-bottom: 24px;
}

.skeleton-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 20px;
  margin-bottom: 24px;
}

.skeleton-card {
  height: 120px;
  background: var(--apple-card);
  border-radius: 16px;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.stat-card {
  background: var(--apple-card);
  border-radius: 16px;
  padding: 24px;
  box-shadow: var(--apple-shadow);
  transition: var(--apple-transition);
  display: flex;
  gap: 16px;
  animation: fadeIn 0.4s ease;
}

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--apple-shadow-lg);
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
  flex-shrink: 0;
}

.stat-content {
  flex: 1;
}

.stat-label {
  font-size: 13px;
  color: var(--apple-text-secondary);
  font-weight: 500;
  margin-bottom: 4px;
}

.stat-value {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-h);
  letter-spacing: -0.02em;
}

.stat-change {
  font-size: 12px;
  font-weight: 500;
  margin-top: 4px;
}

.stat-change.positive { color: var(--apple-green); }
.stat-change.negative { color: var(--apple-red); }

.content-grid {
  display: grid;
  grid-template-columns: 1.2fr 1fr;
  gap: 20px;
  margin-bottom: 24px;
}

.card {
  background: var(--apple-card);
  border-radius: 16px;
  padding: 24px;
  box-shadow: var(--apple-shadow);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-h);
}

.view-all {
  font-size: 13px;
  color: var(--apple-blue);
  text-decoration: none;
  font-weight: 500;
}

.view-all:hover { text-decoration: underline; }

.record-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.record-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--apple-bg);
  border-radius: 10px;
  transition: var(--apple-transition);
}

.record-item:hover {
  background: var(--apple-gray);
}

.record-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.record-icon.in { background: rgba(52, 199, 89, 0.15); color: var(--apple-green); }
.record-icon.out { background: rgba(255, 149, 0, 0.15); color: var(--apple-orange); }

.record-info {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.record-info .pipe-id {
  font-weight: 600;
  font-size: 14px;
}

.record-info .qty {
  font-size: 12px;
  color: var(--apple-text-secondary);
}

.op-badge {
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
}

.op-badge.in { background: rgba(52, 199, 89, 0.12); color: var(--apple-green); }
.op-badge.out { background: rgba(255, 149, 0, 0.12); color: var(--apple-orange); }

.date { 
  font-size: 12px; 
  color: var(--apple-text-secondary); 
  min-width: 60px;
  text-align: right;
}

.material-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.material-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.material-bar {
  height: 8px;
  background: var(--apple-gray);
  border-radius: 4px;
  overflow: hidden;
}

.material-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.5s ease;
}

.material-info {
  display: flex;
  justify-content: space-between;
}

.material-name { font-weight: 600; font-size: 14px; }
.material-stats-info { font-size: 12px; color: var(--apple-text-secondary); }

.alert-card {
  background: linear-gradient(135deg, #fff3e0 0%, #ffe0b2 100%);
  border-radius: 16px;
  padding: 20px;
  box-shadow: var(--apple-shadow);
}

@media (prefers-color-scheme: dark) {
  .alert-card {
    background: linear-gradient(135deg, #3d2e1e 0%, #4a3728 100%);
  }
}

.alert-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: var(--apple-orange);
  margin-bottom: 12px;
}

.alert-count {
  margin-left: auto;
  background: var(--apple-orange);
  color: white;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 12px;
}

.alert-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.alert-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.5);
  border-radius: 8px;
}

@media (prefers-color-scheme: dark) {
  .alert-item {
    background: rgba(0, 0, 0, 0.2);
  }
}

.alert-item .pipe-id { font-weight: 600; font-size: 14px; }
.alert-item .pipe-spec { flex: 1; font-size: 12px; color: var(--apple-text-secondary); }
.warning-qty { color: var(--apple-red); font-weight: 700; }

.alert-action {
  display: block;
  text-align: center;
  color: var(--apple-orange);
  font-weight: 500;
  font-size: 14px;
  text-decoration: none;
}

.alert-action:hover { text-decoration: underline; }

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--apple-text-secondary);
  font-size: 14px;
  padding: 40px 0;
}

.toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  padding: 12px 24px;
  border-radius: 12px;
  font-size: 14px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 12px;
  z-index: 1000;
  animation: slideUp 0.3s ease;
  box-shadow: var(--apple-shadow-lg);
}

.toast.error { background: var(--apple-red); color: white; }
.toast button { background: none; color: inherit; font-size: 18px; padding: 0; opacity: 0.7; border: none; cursor: pointer; }

@keyframes slideUp {
  from { opacity: 0; transform: translateX(-50%) translateY(20px); }
  to { opacity: 1; transform: translateX(-50%) translateY(0); }
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

@media (max-width: 1024px) {
  .content-grid {
    grid-template-columns: 1fr;
  }
}
</style>
