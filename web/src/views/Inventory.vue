<template>
  <div class="page">
    <h1 class="page-title">库存查询</h1>

    <div class="toolbar">
      <input v-model="filters.search" placeholder="搜索钢管编号、材质..." class="search-input" @input="debounceSearch" />
      <select v-model="filters.status" @change="fetchPipes">
        <option value="">全部状态</option>
        <option value="在库">在库</option>
        <option value="已出库">已出库</option>
      </select>
      <input v-model="filters.material" placeholder="材质" class="sm-input" @input="debounceSearch" />
      <button @click="fetchPipes" class="btn-secondary">刷新</button>
    </div>

    <div class="card table-card">
      <table class="apple-table" v-if="pipes.length">
        <thead>
          <tr>
            <th>钢管编号</th>
            <th>直径(mm)</th>
            <th>壁厚(mm)</th>
            <th>长度(m)</th>
            <th>材质</th>
            <th>数量</th>
            <th>存放位置</th>
            <th>状态</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="p in pipes" :key="p.id">
            <td class="fw-600">{{ p.pipe_id }}</td>
            <td>{{ p.diameter.toFixed(2) }}</td>
            <td>{{ p.thickness.toFixed(2) }}</td>
            <td>{{ p.length.toFixed(2) }}</td>
            <td>{{ p.material }}</td>
            <td :class="{ 'qty-low': p.quantity <= 10 }">{{ p.quantity }}</td>
            <td>{{ p.location || '-' }}</td>
            <td>
              <span class="status-badge" :class="p.status === '在库' ? 'in' : 'out'">{{ p.status }}</span>
            </td>
            <td>
              <button class="btn-icon" @click="editPipe(p)">编辑</button>
              <button class="btn-icon btn-danger" @click="deletePipe(p.pipe_id)">删除</button>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-else class="empty">暂无数据</div>
    </div>

    <div class="pagination" v-if="total > 0">
      <span class="page-info">共 {{ total }} 条记录</span>
      <button :disabled="page <= 1" @click="page--; fetchPipes()">上一页</button>
      <span>第 {{ page }} / {{ totalPages }} 页</span>
      <button :disabled="page >= totalPages" @click="page++; fetchPipes()">下一页</button>
    </div>

    <div v-if="showEdit" class="modal-overlay" @click.self="showEdit = false">
      <div class="modal">
        <h2 class="modal-title">编辑钢管信息</h2>
        <div class="form-grid">
          <div class="form-group">
            <label>直径</label>
            <input v-model.number="editForm.diameter" type="number" step="0.01" />
          </div>
          <div class="form-group">
            <label>壁厚</label>
            <input v-model.number="editForm.thickness" type="number" step="0.01" />
          </div>
          <div class="form-group">
            <label>长度</label>
            <input v-model.number="editForm.length" type="number" step="0.01" />
          </div>
          <div class="form-group">
            <label>材质</label>
            <input v-model="editForm.material" />
          </div>
          <div class="form-group">
            <label>数量</label>
            <input v-model.number="editForm.quantity" type="number" />
          </div>
          <div class="form-group">
            <label>状态</label>
            <select v-model="editForm.status">
              <option value="在库">在库</option>
              <option value="已出库">已出库</option>
            </select>
          </div>
        </div>
        <div class="modal-actions">
          <button @click="saveEdit" class="btn-primary">保存</button>
          <button @click="showEdit = false" class="btn-secondary">取消</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { pipesAPI } from '../api'

const pipes = ref([])
const total = ref(0)
const page = ref(1)
const perPage = 20
const totalPages = computed(() => Math.max(1, Math.ceil(total.value / perPage)))

const filters = reactive({ search: '', status: '', material: '' })
const showEdit = ref(false)
const editForm = reactive({ id: null, pipe_id: '', diameter: 0, thickness: 0, length: 0, material: '', quantity: 0, status: '在库' })

let searchTimer = null
function debounceSearch() {
  clearTimeout(searchTimer)
  searchTimer = setTimeout(() => { page.value = 1; fetchPipes() }, 400)
}

async function fetchPipes() {
  try {
    const { data } = await pipesAPI.list({
      page: page.value, per_page: perPage,
      search: filters.search || undefined,
      status: filters.status || undefined,
      material: filters.material || undefined,
    })
    pipes.value = data.pipes
    total.value = data.total
  } catch (e) { console.error(e) }
}

function editPipe(p) {
  Object.assign(editForm, { id: p.id, pipe_id: p.pipe_id, diameter: p.diameter, thickness: p.thickness, length: p.length, material: p.material, quantity: p.quantity, status: p.status })
  showEdit.value = true
}

async function saveEdit() {
  try {
    await pipesAPI.update(editForm.pipe_id, editForm)
    showEdit.value = false
    fetchPipes()
  } catch (e) { console.error(e) }
}

async function deletePipe(id) {
  if (!confirm(`确定删除钢管 "${id}" 吗？`)) return
  try {
    await pipesAPI.delete(id)
    fetchPipes()
  } catch (e) { console.error(e) }
}

onMounted(fetchPipes)
</script>

<style scoped>
.page-title { font-size: 34px; font-weight: 700; letter-spacing: -0.02em; margin-bottom: 28px; }

.toolbar {
  display: flex; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; align-items: center;
}

.search-input {
  flex: 1; min-width: 200px; padding: 10px 16px; border: 1px solid var(--apple-border);
  border-radius: 980px; font-size: 15px; background: var(--apple-card);
}

.search-input:focus { border-color: var(--apple-blue); box-shadow: 0 0 0 3px rgba(0,113,227,0.15); }

.sm-input {
  padding: 10px 14px; border: 1px solid var(--apple-border); border-radius: var(--apple-radius-sm);
  font-size: 14px; background: var(--apple-card); width: 120px;
}

select {
  padding: 10px 14px; border: 1px solid var(--apple-border); border-radius: var(--apple-radius-sm);
  font-size: 14px; background: var(--apple-card);
}

.btn-secondary {
  padding: 10px 20px; border-radius: 980px; font-size: 14px; font-weight: 500;
  background: var(--apple-gray); color: var(--apple-text); transition: var(--apple-transition);
}
.btn-secondary:hover { background: #e8e8ed; }

.card { background: var(--apple-card); border-radius: var(--apple-radius); box-shadow: var(--apple-shadow); overflow: hidden; }
.table-card { padding: 0; }

.apple-table { width: 100%; border-collapse: collapse; }
.apple-table th {
  text-align: left; padding: 12px 16px; font-size: 12px; font-weight: 600;
  color: var(--apple-text-secondary); text-transform: uppercase; letter-spacing: 0.05em;
  background: var(--apple-gray); border-bottom: 1px solid var(--apple-border);
}
.apple-table td {
  padding: 12px 16px; font-size: 14px; border-bottom: 1px solid var(--apple-border);
}
.apple-table tr:last-child td { border-bottom: none; }
.apple-table tr:hover { background: rgba(0,0,0,0.02); }

.fw-600 { font-weight: 600; }
.qty-low { color: var(--apple-red); font-weight: 700; }

.status-badge {
  padding: 3px 10px; border-radius: 20px; font-size: 12px; font-weight: 600;
}
.status-badge.in { background: rgba(52,199,89,0.12); color: var(--apple-green); }
.status-badge.out { background: rgba(255,59,48,0.12); color: var(--apple-red); }

.btn-icon {
  padding: 4px 10px; border-radius: 6px; font-size: 13px; font-weight: 500;
  background: transparent; color: var(--apple-blue); margin-right: 4px;
}
.btn-icon:hover { background: rgba(0,113,227,0.1); }
.btn-danger { color: var(--apple-red); }
.btn-danger:hover { background: rgba(255,59,48,0.1); }

.pagination {
  display: flex; align-items: center; gap: 16px; margin-top: 20px;
  justify-content: center; font-size: 14px; color: var(--apple-text-secondary);
}
.pagination button {
  padding: 8px 16px; border-radius: 980px; font-size: 13px; font-weight: 500;
  background: var(--apple-card); border: 1px solid var(--apple-border);
}
.pagination button:disabled { opacity: 0.3; cursor: not-allowed; }

.empty { padding: 40px; text-align: center; color: var(--apple-text-secondary); font-size: 15px; }

.modal-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.4); backdrop-filter: blur(4px);
  display: flex; align-items: center; justify-content: center; z-index: 200;
}
.modal {
  background: var(--apple-card); border-radius: var(--apple-radius); padding: 32px;
  width: 500px; max-width: 90vw; box-shadow: var(--apple-shadow-lg);
}
.modal-title { font-size: 20px; font-weight: 600; margin-bottom: 20px; }
.form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
.form-group { display: flex; flex-direction: column; gap: 6px; }
.form-group label { font-size: 13px; font-weight: 500; color: var(--apple-text-secondary); }
.form-group input, .form-group select {
  padding: 10px 14px; border: 1px solid var(--apple-border); border-radius: var(--apple-radius-sm);
  font-size: 15px; background: var(--apple-gray);
}
.modal-actions { display: flex; gap: 12px; margin-top: 24px; justify-content: flex-end; }
.btn-primary {
  background: var(--apple-blue); color: white; padding: 10px 24px; border-radius: 980px;
  font-size: 14px; font-weight: 500;
}
</style>
