<template>
  <div class="dashboard">
    <h1 class="page-title">首页概览</h1>

    <div class="stats-grid" v-if="stats">
      <div class="stat-card" :style="{ '--accent': '#0071e3' }">
        <div class="stat-label">钢管种类</div>
        <div class="stat-value">{{ stats.total_types }}</div>
      </div>
      <div class="stat-card" :style="{ '--accent': '#34c759' }">
        <div class="stat-label">库存总量</div>
        <div class="stat-value">{{ stats.total_quantity }}</div>
      </div>
      <div class="stat-card" :style="{ '--accent': '#5856d6' }">
        <div class="stat-label">入库总数</div>
        <div class="stat-value">{{ stats.total_in }}</div>
      </div>
      <div class="stat-card" :style="{ '--accent': '#ff3b30' }">
        <div class="stat-label">出库总数</div>
        <div class="stat-value">{{ stats.total_out }}</div>
      </div>
    </div>
    <div v-else class="skeleton-grid">
      <div v-for="i in 4" :key="i" class="skeleton-card"></div>
    </div>

    <div class="content-grid">
      <div class="card">
        <h2 class="card-title">最近操作记录</h2>
        <div class="record-list" v-if="recentRecords.length">
          <div v-for="r in recentRecords" :key="r.id" class="record-item">
            <span class="op-badge" :class="r.operation_type === '入库' ? 'in' : 'out'">{{ r.operation_type }}</span>
            <span class="pipe-id">{{ r.pipe_id }}</span>
            <span class="qty">数量: {{ r.quantity }}</span>
            <span class="date">{{ r.operation_date }}</span>
          </div>
        </div>
        <div v-else class="empty">暂无操作记录</div>
      </div>

      <div class="card">
        <h2 class="card-title">按材质统计</h2>
        <div class="material-list" v-if="materialStats.length">
          <div v-for="m in materialStats" :key="m.material" class="material-item">
            <span class="material-name">{{ m.material }}</span>
            <span class="material-info">种类: {{ m.type_count }} | 数量: {{ m.total_quantity }}</span>
          </div>
        </div>
        <div v-else class="empty">暂无数据</div>
      </div>

      <div class="card" v-if="lowStock.length">
        <h2 class="card-title" style="color: var(--apple-orange)">⚠ 库存预警</h2>
        <div class="warning-list">
          <div v-for="p in lowStock" :key="p.id" class="warning-item">
            <span>{{ p.pipe_id }}</span>
            <span class="warning-qty">{{ p.quantity }}</span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="errorMsg" class="toast error">
      {{ errorMsg }}
      <button @click="errorMsg = ''">×</button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { statsAPI, recordsAPI } from '../api'

const stats = ref(null)
const recentRecords = ref([])
const materialStats = ref([])
const lowStock = ref([])
const errorMsg = ref('')

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
    recentRecords.value = rr.data.slice(0, 10)
  } catch (e) {
    errorMsg.value = e.message || '加载数据失败'
  }
})
</script>

<style scoped>
.page-title {
  font-size: 34px;
  font-weight: 700;
  letter-spacing: -0.02em;
  margin-bottom: 28px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-bottom: 28px;
}

.skeleton-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-bottom: 28px;
}

.skeleton-card {
  height: 100px;
  background: var(--apple-card);
  border-radius: var(--apple-radius);
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.stat-card {
  background: var(--apple-card);
  border-radius: var(--apple-radius);
  padding: 24px;
  box-shadow: var(--apple-shadow);
  transition: var(--apple-transition);
}

.stat-card:hover {
  box-shadow: var(--apple-shadow-lg);
  transform: translateY(-2px);
}

.stat-label {
  font-size: 13px;
  color: var(--apple-text-secondary);
  font-weight: 500;
  margin-bottom: 8px;
}

.stat-value {
  font-size: 40px;
  font-weight: 700;
  color: var(--accent);
  letter-spacing: -0.03em;
}

.content-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.card {
  background: var(--apple-card);
  border-radius: var(--apple-radius);
  padding: 24px;
  box-shadow: var(--apple-shadow);
}

.card-title {
  font-size: 17px;
  font-weight: 600;
  margin-bottom: 16px;
  letter-spacing: -0.01em;
}

.record-list, .material-list, .warning-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.record-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 0;
  border-bottom: 1px solid var(--apple-border);
  font-size: 14px;
}

.record-item:last-child { border-bottom: none; }

.op-badge {
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
}

.op-badge.in { background: rgba(52, 199, 89, 0.12); color: var(--apple-green); }
.op-badge.out { background: rgba(255, 59, 48, 0.12); color: var(--apple-red); }

.pipe-id { font-weight: 600; }
.qty { color: var(--apple-text-secondary); }
.date { color: var(--apple-text-secondary); margin-left: auto; font-size: 13px; }

.material-item {
  display: flex;
  justify-content: space-between;
  padding: 10px 0;
  border-bottom: 1px solid var(--apple-border);
  font-size: 14px;
}

.material-item:last-child { border-bottom: none; }
.material-name { font-weight: 600; }
.material-info { color: var(--apple-text-secondary); }

.warning-item {
  display: flex;
  justify-content: space-between;
  padding: 10px 0;
  border-bottom: 1px solid var(--apple-border);
  font-size: 14px;
}

.warning-qty {
  color: var(--apple-red);
  font-weight: 700;
}

.empty {
  color: var(--apple-text-secondary);
  font-size: 14px;
  padding: 20px 0;
  text-align: center;
}

.toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  padding: 12px 24px;
  border-radius: 980px;
  font-size: 14px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 12px;
  z-index: 1000;
  animation: slideUp 0.3s ease;
}

.toast.error { background: var(--apple-red); color: white; }
.toast button { background: none; color: inherit; font-size: 18px; padding: 0; opacity: 0.7; }

@keyframes slideUp {
  from { opacity: 0; transform: translateX(-50%) translateY(20px); }
  to { opacity: 1; transform: translateX(-50%) translateY(0); }
}
</style>
