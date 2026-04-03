<template>
  <div class="page">
    <h1 class="page-title">数据统计</h1>

    <div class="stats-grid" v-if="stats">
      <div class="stat-card" :style="{ '--accent': '#0071e3' }">
        <div class="stat-label">总种类</div>
        <div class="stat-value">{{ stats.total_types }}</div>
      </div>
      <div class="stat-card" :style="{ '--accent': '#34c759' }">
        <div class="stat-label">总数量</div>
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

    <div class="content-grid">
      <div class="card">
        <h2 class="card-title">按材质分类</h2>
        <div class="chart-bars" v-if="materialStats.length">
          <div v-for="m in materialStats" :key="m.material" class="bar-item">
            <div class="bar-label">{{ m.material }}</div>
            <div class="bar-track">
              <div class="bar-fill" :style="{ width: barWidth(m.total_quantity) + '%' }">
                <span class="bar-value">{{ m.total_quantity }}</span>
              </div>
            </div>
            <div class="bar-count">{{ m.type_count }} 种</div>
          </div>
        </div>
        <div v-else class="empty">暂无数据</div>
      </div>

      <div class="card">
        <h2 class="card-title">入库 vs 出库</h2>
        <div class="comparison" v-if="stats">
          <div class="compare-item">
            <div class="compare-label">入库</div>
            <div class="compare-bar-track">
              <div class="compare-bar-fill" :style="{ width: compareWidth(stats.total_in) + '%', background: 'var(--apple-green)' }">
                <span>{{ stats.total_in }}</span>
              </div>
            </div>
          </div>
          <div class="compare-item">
            <div class="compare-label">出库</div>
            <div class="compare-bar-track">
              <div class="compare-bar-fill" :style="{ width: compareWidth(stats.total_out) + '%', background: 'var(--apple-red)' }">
                <span>{{ stats.total_out }}</span>
              </div>
            </div>
          </div>
          <div class="compare-ratio" v-if="stats.total_in + stats.total_out > 0">
            入库占比: {{ ((stats.total_in / (stats.total_in + stats.total_out)) * 100).toFixed(1) }}%
          </div>
        </div>
      </div>
    </div>

    <div class="card" style="margin-top: 16px;">
      <h2 class="card-title" style="color: var(--apple-orange);">⚠ 库存预警</h2>
      <div class="threshold-row">
        <label>预警阈值:</label>
        <input v-model.number="threshold" type="number" min="1" max="1000" class="threshold-input" />
        <button @click="fetchLowStock" class="btn-secondary">查询</button>
      </div>
      <div class="table-wrap" v-if="lowStock.length">
        <table class="apple-table">
          <thead><tr><th>钢管编号</th><th>材质</th><th>当前库存</th><th>存放位置</th></tr></thead>
          <tbody>
            <tr v-for="p in lowStock" :key="p.id">
              <td class="fw-600">{{ p.pipe_id }}</td>
              <td>{{ p.material }}</td>
              <td class="qty-low">{{ p.quantity }}</td>
              <td>{{ p.location || '-' }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <div v-else class="empty">所有库存充足，暂无预警项目</div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { statsAPI } from '../api'

const stats = ref(null)
const materialStats = ref([])
const lowStock = ref([])
const threshold = ref(10)

const maxMaterialQty = ref(1)
const maxCompare = ref(1)

function barWidth(qty) {
  return (qty / maxMaterialQty.value) * 100
}

function compareWidth(qty) {
  return maxCompare.value > 0 ? (qty / maxCompare.value) * 100 : 0
}

async function fetchData() {
  try {
    const [s, ms] = await Promise.all([statsAPI.overview(), statsAPI.byMaterial()])
    stats.value = s.data
    materialStats.value = ms.data
    if (ms.data.length) maxMaterialQty.value = Math.max(...ms.data.map(m => m.total_quantity), 1)
    maxCompare.value = Math.max(s.data.total_in, s.data.total_out, 1)
  } catch (e) { console.error(e) }
}

async function fetchLowStock() {
  try {
    const { data } = await statsAPI.lowStock(threshold.value)
    lowStock.value = data
  } catch (e) { console.error(e) }
}

onMounted(() => { fetchData(); fetchLowStock() })
</script>

<style scoped>
.page-title { font-size: 34px; font-weight: 700; letter-spacing: -0.02em; margin-bottom: 28px; }
.stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; margin-bottom: 28px; }
.stat-card { background: var(--apple-card); border-radius: var(--apple-radius); padding: 24px; box-shadow: var(--apple-shadow); transition: var(--apple-transition); }
.stat-card:hover { box-shadow: var(--apple-shadow-lg); transform: translateY(-2px); }
.stat-label { font-size: 13px; color: var(--apple-text-secondary); font-weight: 500; margin-bottom: 8px; }
.stat-value { font-size: 40px; font-weight: 700; color: var(--accent); letter-spacing: -0.03em; }
.content-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
.card { background: var(--apple-card); border-radius: var(--apple-radius); padding: 24px; box-shadow: var(--apple-shadow); }
.card-title { font-size: 17px; font-weight: 600; margin-bottom: 16px; letter-spacing: -0.01em; }

.chart-bars { display: flex; flex-direction: column; gap: 14px; }
.bar-item { display: flex; align-items: center; gap: 12px; }
.bar-label { width: 80px; font-size: 14px; font-weight: 500; text-align: right; }
.bar-track { flex: 1; height: 28px; background: var(--apple-gray); border-radius: 14px; overflow: hidden; }
.bar-fill { height: 100%; background: var(--apple-blue); border-radius: 14px; display: flex; align-items: center; justify-content: flex-end; padding-right: 10px; min-width: 40px; transition: width 0.6s ease; }
.bar-value { color: white; font-size: 12px; font-weight: 600; }
.bar-count { width: 50px; font-size: 13px; color: var(--apple-text-secondary); }

.comparison { display: flex; flex-direction: column; gap: 20px; }
.compare-item { display: flex; align-items: center; gap: 12px; }
.compare-label { width: 50px; font-size: 14px; font-weight: 500; }
.compare-bar-track { flex: 1; height: 32px; background: var(--apple-gray); border-radius: 16px; overflow: hidden; }
.compare-bar-fill { height: 100%; border-radius: 16px; display: flex; align-items: center; padding-left: 14px; color: white; font-size: 14px; font-weight: 600; min-width: 50px; transition: width 0.6s ease; }
.compare-ratio { font-size: 14px; color: var(--apple-text-secondary); text-align: center; padding-top: 8px; border-top: 1px solid var(--apple-border); }

.threshold-row { display: flex; align-items: center; gap: 12px; margin-bottom: 16px; }
.threshold-row label { font-size: 14px; font-weight: 500; }
.threshold-input { width: 80px; padding: 8px 12px; border: 1px solid var(--apple-border); border-radius: var(--apple-radius-sm); font-size: 14px; }
.btn-secondary { padding: 8px 16px; border-radius: 980px; font-size: 13px; font-weight: 500; background: var(--apple-gray); }

.apple-table { width: 100%; border-collapse: collapse; }
.apple-table th { text-align: left; padding: 10px 14px; font-size: 12px; font-weight: 600; color: var(--apple-text-secondary); background: var(--apple-gray); }
.apple-table td { padding: 10px 14px; font-size: 14px; border-bottom: 1px solid var(--apple-border); }
.fw-600 { font-weight: 600; }
.qty-low { color: var(--apple-red); font-weight: 700; }
.empty { padding: 30px; text-align: center; color: var(--apple-text-secondary); font-size: 14px; }
</style>
