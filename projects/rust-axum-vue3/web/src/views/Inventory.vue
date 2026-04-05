<template>
  <div class="page">
    <div class="toolbar">
      <input v-model="filters.search" placeholder="搜索钢管编号、材质..." class="search-input" @input="debounceSearch" />
      <select v-model="filters.status" @change="fetchPipes">
        <option value="">全部状态</option>
        <option value="在库">在库</option>
        <option value="已出库">已出库</option>
      </select>
      <input v-model="filters.material" placeholder="材质" class="sm-input" @input="debounceSearch" />
      <button @click="fetchPipes" class="btn-secondary" :disabled="loading">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M23 4v6h-6M1 20v-6h6M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>
        刷新
      </button>
    </div>

    <div class="toolbar-actions">
      <div class="action-group">
        <button @click="showExportMenu = !showExportMenu" class="btn-primary" :disabled="loading">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7,10 12,15 17,10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
          导出
        </button>
        <div v-if="showExportMenu" class="dropdown-menu">
          <button @click="exportCSV">导出 CSV</button>
          <button @click="exportExcel">导出 Excel</button>
          <button @click="exportFiltered">导出筛选结果</button>
        </div>
      </div>
      
      <div class="action-group">
        <label class="btn-import">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17,8 12,3 7,8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
          导入
          <input type="file" accept=".csv,.xlsx,.xls" @change="handleImport" hidden />
        </label>
      </div>
      
      <button @click="batchDelete" class="btn-danger" :disabled="selectedIds.length === 0">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3,6 5,6 21,6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
        批量删除 ({{ selectedIds.length }})
      </button>
    </div>

    <div v-if="loading" class="loading-container">
      <div class="spinner"></div>
      <span>加载中...</span>
    </div>

    <div class="card table-card" v-else-if="pipes.length">
      <table class="apple-table">
        <thead>
          <tr>
            <th class="col-check"><input type="checkbox" @change="toggleAll" :checked="allSelected" /></th>
            <th>钢管编号</th>
            <th>直径(mm)</th>
            <th>壁厚(mm)</th>
            <th>长度(m)</th>
            <th>材质</th>
            <th>数量</th>
            <th>炉号</th>
            <th>热处理批号</th>
            <th>取样号</th>
            <th>原料架</th>
            <th>存放位置</th>
            <th>状态</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="p in pipes" :key="p.id">
            <td class="col-check"><input type="checkbox" :value="p.pipe_id" v-model="selectedIds" /></td>
            <td class="fw-600">{{ p.pipe_id }}</td>
            <td>{{ p.diameter.toFixed(2) }}</td>
            <td>{{ p.thickness.toFixed(2) }}</td>
            <td>{{ p.length.toFixed(2) }}</td>
            <td>{{ p.material }}</td>
            <td :class="{ 'qty-low': p.quantity <= 10 }">{{ p.quantity }}</td>
            <td>{{ p.furnace_number || '-' }}</td>
            <td>{{ p.heat_treatment_batch || '-' }}</td>
            <td>{{ p.sample_number || '-' }}</td>
            <td>{{ p.material_rack || '-' }}</td>
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
    </div>
    <div v-else class="card empty">暂无数据</div>

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
          <div class="form-group">
            <label>炉号</label>
            <input v-model="editForm.furnace_number" />
          </div>
          <div class="form-group">
            <label>热处理批号</label>
            <input v-model="editForm.heat_treatment_batch" />
          </div>
          <div class="form-group">
            <label>取样号</label>
            <input v-model="editForm.sample_number" />
          </div>
          <div class="form-group">
            <label>投产支数</label>
            <input v-model.number="editForm.production_count" type="number" />
          </div>
          <div class="form-group">
            <label>原料架</label>
            <input v-model="editForm.material_rack" />
          </div>
          <div class="form-group">
            <label>存放位置</label>
            <input v-model="editForm.location" />
          </div>
        </div>
        <div class="form-group full">
          <label>备注</label>
          <textarea v-model="editForm.remarks" rows="2"></textarea>
        </div>
        <div class="modal-actions">
          <button @click="saveEdit" class="btn-primary" :disabled="saving">
            {{ saving ? '保存中...' : '保存' }}
          </button>
          <button @click="showEdit = false" class="btn-secondary">取消</button>
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
import { ref, reactive, computed, onMounted } from 'vue'
import { pipesAPI, exportAPI, importAPI } from '../api'

const pipes = ref([])
const total = ref(0)
const page = ref(1)
const perPage = 20
const totalPages = computed(() => Math.max(1, Math.ceil(total.value / perPage)))
const loading = ref(false)
const saving = ref(false)
const errorMsg = ref('')
const showExportMenu = ref(false)

const filters = reactive({ search: '', status: '', material: '' })
const showEdit = ref(false)
const editForm = reactive({ 
  id: null, pipe_id: '', diameter: 0, thickness: 0, length: 0, 
  material: '', quantity: 0, status: '在库',
  furnace_number: '', heat_treatment_batch: '', sample_number: '',
  production_count: null, material_rack: '', location: '', remarks: ''
})
const selectedIds = ref([])
const allSelected = computed(() => pipes.value.length > 0 && selectedIds.value.length === pipes.value.length)

let searchTimer = null
function debounceSearch() {
  clearTimeout(searchTimer)
  searchTimer = setTimeout(() => { page.value = 1; fetchPipes() }, 400)
}

async function exportCSV() {
  showExportMenu.value = false
  try {
    loading.value = true
    const { data } = await exportAPI.inventory()
    downloadBlob(data, 'inventory.csv', 'text/csv')
  } catch (e) {
    errorMsg.value = e.message || '导出失败'
  } finally {
    loading.value = false
  }
}

async function exportExcel() {
  showExportMenu.value = false
  try {
    loading.value = true
    const { data } = await exportAPI.inventoryExcel()
    downloadBlob(data, 'inventory.xlsx', 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet')
  } catch (e) {
    errorMsg.value = e.message || '导出失败'
  } finally {
    loading.value = false
  }
}

async function exportFiltered() {
  showExportMenu.value = false
  try {
    loading.value = true
    const params = {}
    if (filters.search) params.search = filters.search
    if (filters.material) params.material = filters.material
    if (filters.status) params.status = filters.status
    const { data } = await pipesAPI.batchExport ? await pipesAPI.batchExport(params) : await exportAPI.inventory()
    downloadBlob(data, 'inventory_filtered.csv', 'text/csv')
  } catch (e) {
    errorMsg.value = e.message || '导出失败'
  } finally {
    loading.value = false
  }
}

async function handleImport(e) {
  const file = e.target.files[0]
  if (!file) return
  
  const isExcel = file.name.match(/\.(xlsx|xls)$/i)
  const isCsv = file.name.endsWith('.csv')
  
  if (!isExcel && !isCsv) {
    errorMsg.value = '请选择 CSV 或 Excel 文件'
    return
  }
  
  try {
    loading.value = true
    if (isExcel) {
      const reader = new FileReader()
      reader.onload = async (event) => {
        const base64 = event.target.result.split(',')[1]
        const { data } = await importAPI.excel({ excel_base64: base64, operator: 'admin' })
        if (data.success > 0) {
          errorMsg.value = ''
          fetchPipes()
        }
        if (data.fail?.length > 0) {
          errorMsg.value = `导入完成，成功 ${data.success} 条，失败 ${data.fail.length} 条`
        }
      }
      reader.readAsDataURL(file)
    } else {
      const reader = new FileReader()
      reader.onload = async (event) => {
        const csv_content = event.target.result
        const { data } = await importAPI.csv({ csv_content, operator: 'admin' })
        if (data.success > 0) {
          errorMsg.value = ''
          fetchPipes()
        }
        if (data.fail?.length > 0) {
          errorMsg.value = `导入完成，成功 ${data.success} 条，失败 ${data.fail.length} 条`
        }
      }
      reader.readAsText(file)
    }
  } catch (e) {
    errorMsg.value = e.message || '导入失败'
  } finally {
    loading.value = false
    e.target.value = ''
  }
}

function downloadBlob(blob, filename, type) {
  const url = URL.createObjectURL(new Blob([blob], { type }))
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.click()
  URL.revokeObjectURL(url)
}

function toggleAll(e) {
  if (e.target.checked) {
    selectedIds.value = pipes.value.map(p => p.pipe_id)
  } else {
    selectedIds.value = []
  }
}

async function batchDelete() {
  if (selectedIds.value.length === 0) return
  if (!confirm(`确定删除选中的 ${selectedIds.value.length} 条记录吗？`)) return
  try {
    const operator = prompt('请输入操作员名称:', 'admin')
    if (!operator) return
    await pipesAPI.batchDelete({ pipe_ids: selectedIds.value, operator })
    selectedIds.value = []
    fetchPipes()
  } catch (e) {
    errorMsg.value = e.message || '批量删除失败'
  }
}

async function fetchPipes() {
  loading.value = true
  try {
    const { data } = await pipesAPI.list({
      page: page.value, per_page: perPage,
      search: filters.search || undefined,
      status: filters.status || undefined,
      material: filters.material || undefined,
    })
    pipes.value = data.pipes
    total.value = data.total
  } catch (e) {
    errorMsg.value = e.message || '加载失败'
  } finally {
    loading.value = false
  }
}

function editPipe(p) {
  Object.assign(editForm, { 
    id: p.id, pipe_id: p.pipe_id, diameter: p.diameter, thickness: p.thickness, 
    length: p.length, material: p.material, quantity: p.quantity, status: p.status,
    furnace_number: p.furnace_number || '', heat_treatment_batch: p.heat_treatment_batch || '',
    sample_number: p.sample_number || '', production_count: p.production_count,
    material_rack: p.material_rack || '', location: p.location || '', remarks: p.remarks || ''
  })
  showEdit.value = true
}

async function saveEdit() {
  saving.value = true
  try {
    await pipesAPI.update(editForm.pipe_id, editForm)
    showEdit.value = false
    fetchPipes()
  } catch (e) {
    errorMsg.value = e.message || '保存失败'
  } finally {
    saving.value = false
  }
}

async function deletePipe(id) {
  if (!confirm(`确定删除钢管 "${id}" 吗？`)) return
  try {
    await pipesAPI.delete(id)
    fetchPipes()
  } catch (e) {
    errorMsg.value = e.message || '删除失败'
  }
}

onMounted(fetchPipes)
</script>

<style scoped>
.page-title { font-size: 34px; font-weight: 700; letter-spacing: -0.02em; margin-bottom: 28px; }

.toolbar, .toolbar-actions {
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
.btn-secondary:disabled { opacity: 0.5; cursor: not-allowed; }

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
.col-check { width: 40px; text-align: center; }
.col-check input { width: 18px; height: 18px; cursor: pointer; }

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

.loading-container {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  padding: 60px 0; gap: 16px; color: var(--apple-text-secondary);
}
.spinner {
  width: 32px; height: 32px; border: 3px solid var(--apple-border);
  border-top-color: var(--apple-blue); border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

.modal-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.4); backdrop-filter: blur(4px);
  display: flex; align-items: center; justify-content: center; z-index: 200;
}
.modal {
  background: var(--apple-card); border-radius: var(--apple-radius); padding: 32px;
  width: 500px; max-width: 90vw; box-shadow: var(--apple-shadow-lg);
}
.modal-title { font-size: 20px; font-weight: 600; margin-bottom: 20px; }
.toolbar-actions {
  display: flex; gap: 12px; margin-bottom: 20px; flex-wrap: wrap;
}

.action-group {
  position: relative;
}

.btn-primary {
  display: flex; align-items: center; gap: 8px;
  background: var(--apple-blue); color: white; padding: 10px 20px; border-radius: 10px;
  font-size: 14px; font-weight: 500; border: none; cursor: pointer;
  transition: var(--apple-transition);
}
.btn-primary:hover { background: var(--apple-blue-hover); }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

.dropdown-menu {
  position: absolute; top: 100%; left: 0; margin-top: 4px;
  background: var(--apple-card); border-radius: 10px;
  box-shadow: var(--apple-shadow-lg); overflow: hidden; z-index: 100;
  min-width: 140px;
}

.dropdown-menu button {
  display: block; width: 100%; padding: 12px 16px;
  border: none; background: none; text-align: left;
  font-size: 14px; color: var(--apple-text);
  cursor: pointer; transition: var(--apple-transition);
}

.dropdown-menu button:hover {
  background: var(--apple-gray);
}

.btn-import {
  display: flex; align-items: center; gap: 8px;
  background: var(--apple-green); color: white; padding: 10px 20px; border-radius: 10px;
  font-size: 14px; font-weight: 500; cursor: pointer;
  transition: var(--apple-transition);
}
.btn-import:hover { background: #2db84d; }

.btn-danger {
  display: flex; align-items: center; gap: 8px;
  background: var(--apple-red); color: white; padding: 10px 20px; border-radius: 10px;
  font-size: 14px; font-weight: 500; border: none; cursor: pointer;
  transition: var(--apple-transition);
}
.btn-danger:hover { background: #e62a1f; }
.btn-danger:disabled { opacity: 0.5; cursor: not-allowed; }

.toast {
  position: fixed; bottom: 24px; left: 50%; transform: translateX(-50%);
  padding: 12px 24px; border-radius: 12px; font-size: 14px; font-weight: 500;
  display: flex; align-items: center; gap: 12px; z-index: 1000;
  animation: slideUp 0.3s ease;
}
.toast.error { background: var(--apple-red); color: white; }
.toast button { background: none; color: inherit; font-size: 18px; padding: 0; opacity: 0.7; border: none; cursor: pointer; }

@keyframes slideUp {
  from { opacity: 0; transform: translateX(-50%) translateY(20px); }
  to { opacity: 1; transform: translateX(-50%) translateY(0); }
}
</style>
