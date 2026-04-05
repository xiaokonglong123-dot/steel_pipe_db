<template>
  <div class="page">
    <h1 class="page-title">出入库记录</h1>

    <div class="toolbar">
      <select v-model="filters.operation_type" @change="fetchRecords">
        <option value="">全部类型</option>
        <option value="入库">入库</option>
        <option value="出库">出库</option>
      </select>
      <input v-model="filters.pipe_id" placeholder="钢管编号" class="sm-input" @input="debounceFetch" />
      <input v-model="filters.start_date" type="date" class="sm-input" @change="fetchRecords" />
      <span>至</span>
      <input v-model="filters.end_date" type="date" class="sm-input" @change="fetchRecords" />
      <button @click="exportRecords" class="btn-primary">导出CSV</button>
    </div>

    <div class="card table-card">
      <table class="apple-table" v-if="records.length">
        <thead>
          <tr>
            <th>钢管编号</th>
            <th>操作类型</th>
            <th>数量</th>
            <th>操作日期</th>
            <th>操作员</th>
            <th>备注</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="r in records" :key="r.id">
            <td class="fw-600">{{ r.pipe_id }}</td>
            <td>
              <span class="op-badge" :class="r.operation_type === '入库' ? 'in' : 'out'">{{ r.operation_type }}</span>
            </td>
            <td>{{ r.quantity }}</td>
            <td>{{ r.operation_date }}</td>
            <td>{{ r.operator }}</td>
            <td>{{ r.remarks || '-' }}</td>
          </tr>
        </tbody>
      </table>
      <div v-else class="empty">暂无记录</div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { recordsAPI, exportAPI } from '../api'

const records = ref([])
const filters = reactive({ operation_type: '', pipe_id: '', start_date: '', end_date: '' })

let timer = null
function debounceFetch() {
  clearTimeout(timer)
  timer = setTimeout(fetchRecords, 400)
}

async function fetchRecords() {
  try {
    const params = {}
    if (filters.operation_type) params.operation_type = filters.operation_type
    if (filters.pipe_id) params.pipe_id = filters.pipe_id
    if (filters.start_date) params.start_date = filters.start_date
    if (filters.end_date) params.end_date = filters.end_date
    const { data } = await recordsAPI.list(params)
    records.value = data
  } catch (e) { console.error(e) }
}

async function exportRecords() {
  try {
    const params = {}
    if (filters.operation_type) params.operation_type = filters.operation_type
    if (filters.pipe_id) params.pipe_id = filters.pipe_id
    const { data } = await exportAPI.records(params)
    const url = URL.createObjectURL(new Blob([data]))
    const a = document.createElement('a')
    a.href = url
    a.download = 'records.csv'
    a.click()
    URL.revokeObjectURL(url)
  } catch (e) { console.error(e) }
}

onMounted(fetchRecords)
</script>

<style scoped>
.page-title { font-size: 34px; font-weight: 700; letter-spacing: -0.02em; margin-bottom: 28px; }
.toolbar { display: flex; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; align-items: center; }
select, .sm-input {
  padding: 10px 14px; border: 1px solid var(--apple-border); border-radius: var(--apple-radius-sm);
  font-size: 14px; background: var(--apple-card);
}
.btn-primary {
  background: var(--apple-blue); color: white; padding: 10px 20px; border-radius: 980px;
  font-size: 14px; font-weight: 500;
}
.card { background: var(--apple-card); border-radius: var(--apple-radius); box-shadow: var(--apple-shadow); overflow: hidden; }
.table-card { padding: 0; }
.apple-table { width: 100%; border-collapse: collapse; }
.apple-table th {
  text-align: left; padding: 12px 16px; font-size: 12px; font-weight: 600;
  color: var(--apple-text-secondary); text-transform: uppercase; letter-spacing: 0.05em;
  background: var(--apple-gray); border-bottom: 1px solid var(--apple-border);
}
.apple-table td { padding: 12px 16px; font-size: 14px; border-bottom: 1px solid var(--apple-border); }
.apple-table tr:last-child td { border-bottom: none; }
.apple-table tr:hover { background: rgba(0,0,0,0.02); }
.fw-600 { font-weight: 600; }
.op-badge { padding: 3px 10px; border-radius: 20px; font-size: 12px; font-weight: 600; }
.op-badge.in { background: rgba(52,199,89,0.12); color: var(--apple-green); }
.op-badge.out { background: rgba(255,59,48,0.12); color: var(--apple-red); }
.empty { padding: 40px; text-align: center; color: var(--apple-text-secondary); font-size: 15px; }
</style>
