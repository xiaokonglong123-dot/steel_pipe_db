<template>
  <div class="page">
    <div class="card">
      <h2 class="form-title">新建热处理工单</h2>
      <form @submit.prevent="submitOrder" class="form">
        <div class="form-grid">
          <div class="form-group">
            <label>工单号 <span class="required">*</span></label>
            <input v-model="orderForm.order_number" placeholder="请输入工单号" required />
          </div>
          <div class="form-group">
            <label>钢管编号 <span class="required">*</span></label>
            <input v-model="orderForm.pipe_id" placeholder="请输入钢管编号" required />
          </div>
          <div class="form-group">
            <label>炉号 <span class="required">*</span></label>
            <input v-model="orderForm.furnace_number" placeholder="请输入炉号" required />
          </div>
          <div class="form-group">
            <label>热处理类型 <span class="required">*</span></label>
            <select v-model="orderForm.heat_treatment_type" required>
              <option value="">请选择</option>
              <option value="正火">正火</option>
              <option value="淬火">淬火</option>
              <option value="回火">回火</option>
              <option value="退火">退火</option>
              <option value="调质">调质</option>
            </select>
          </div>
          <div class="form-group">
            <label>冷却方式</label>
            <input v-model="orderForm.cooling_method" placeholder="如：水冷、油冷" />
          </div>
          <div class="form-group">
            <label>操作员 <span class="required">*</span></label>
            <input v-model="orderForm.operator" placeholder="操作员" required />
          </div>
        </div>
        <div class="form-group full">
          <label>工艺参数</label>
          <textarea v-model="orderForm.process_parameters" rows="2" placeholder="温度曲线、保温时间等"></textarea>
        </div>
        <div class="form-actions">
          <button type="submit" class="btn-primary" :disabled="loading">
            {{ loading ? '提交中...' : '创建工单' }}
          </button>
        </div>
      </form>
    </div>

    <div class="card" style="margin-top: 24px;">
      <h2 class="form-title">热处理工单列表</h2>
      <div class="table-container">
        <table class="apple-table">
          <thead>
            <tr>
              <th>工单号</th>
              <th>钢管编号</th>
              <th>炉号</th>
              <th>类型</th>
              <th>状态</th>
              <th>开始时间</th>
              <th>操作员</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="order in orders" :key="order.id">
              <td>{{ order.order_number }}</td>
              <td>{{ order.pipe_id }}</td>
              <td>{{ order.furnace_number }}</td>
              <td>{{ order.heat_treatment_type }}</td>
              <td><span :class="['status-badge', order.status]">{{ order.status }}</span></td>
              <td>{{ order.start_time }}</td>
              <td>{{ order.operator }}</td>
              <td>
                <button v-if="order.status === '进行中'" @click="completeOrder(order.id)" class="btn-success btn-sm">完成</button>
              </td>
            </tr>
            <tr v-if="orders.length === 0">
              <td colspan="8" style="text-align: center; color: var(--apple-text-secondary);">暂无工单</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div v-if="message" :class="['toast', messageType]">
      {{ message }}
      <button @click="message = ''">×</button>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { heatTreatmentAPI } from '../api'

const orderForm = reactive({
  order_number: '',
  pipe_id: '',
  furnace_number: '',
  heat_treatment_type: '',
  process_parameters: '',
  cooling_method: '',
  operator: '',
})

const orders = ref([])
const loading = ref(false)
const message = ref('')
const messageType = ref('')

async function loadOrders() {
  try {
    const res = await heatTreatmentAPI.list()
    orders.value = res.data
  } catch (e) {
    showMessage(e.message || '加载失败', 'error')
  }
}

async function submitOrder() {
  loading.value = true
  try {
    await heatTreatmentAPI.create({
      order_number: orderForm.order_number,
      pipe_id: orderForm.pipe_id,
      furnace_number: orderForm.furnace_number,
      heat_treatment_type: orderForm.heat_treatment_type,
      process_parameters: orderForm.process_parameters || null,
      operator: orderForm.operator,
      cooling_method: orderForm.cooling_method || null,
      remarks: null,
    })
    showMessage('工单创建成功！', 'success')
    Object.keys(orderForm).forEach(k => orderForm[k] = '')
    loadOrders()
  } catch (e) {
    showMessage(e.message || '创建失败', 'error')
  } finally {
    loading.value = false
  }
}

async function completeOrder(id) {
  try {
    await heatTreatmentAPI.updateStatus(id, { status: '已完成' })
    showMessage('工单已完成', 'success')
    loadOrders()
  } catch (e) {
    showMessage(e.message || '操作失败', 'error')
  }
}

function showMessage(msg, type) {
  message.value = msg
  messageType.value = type
  setTimeout(() => { message.value = '' }, 3000)
}

onMounted(() => {
  loadOrders()
})
</script>

<style scoped>
.card {
  background: var(--apple-card);
  border-radius: 16px;
  padding: 32px;
  box-shadow: var(--apple-shadow);
}

.form-title {
  font-size: 20px;
  font-weight: 600;
  margin-bottom: 24px;
  color: var(--text-h);
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group.full {
  margin-top: 20px;
}

.required {
  color: var(--apple-red);
}

label {
  font-size: 13px;
  font-weight: 500;
  color: var(--apple-text-secondary);
}

input, select, textarea {
  padding: 12px 14px;
  border: 1px solid var(--apple-border);
  border-radius: 10px;
  font-size: 15px;
  background: var(--apple-gray);
  transition: var(--apple-transition);
}

input:focus, select:focus, textarea:focus {
  border-color: var(--apple-blue);
  background: white;
  box-shadow: 0 0 0 3px rgba(0, 113, 227, 0.15);
}

.form-actions {
  margin-top: 28px;
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--apple-blue);
  color: white;
  padding: 14px 32px;
  border-radius: 12px;
  font-size: 15px;
  font-weight: 600;
  border: none;
  cursor: pointer;
  transition: var(--apple-transition);
}

.btn-primary:hover {
  background: var(--apple-blue-hover);
  transform: translateY(-1px);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-success {
  background: var(--apple-green);
  color: white;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 12px;
  border: none;
  cursor: pointer;
}

.btn-sm {
  padding: 4px 8px;
  font-size: 11px;
}

.table-container {
  overflow-x: auto;
}

.apple-table {
  width: 100%;
  border-collapse: collapse;
}

.apple-table th {
  text-align: left;
  padding: 12px 16px;
  font-size: 12px;
  font-weight: 600;
  color: var(--apple-text-secondary);
  border-bottom: 1px solid var(--apple-border);
}

.apple-table td {
  padding: 12px 16px;
  font-size: 14px;
  border-bottom: 1px solid var(--apple-border);
}

.status-badge {
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
}

.status-badge.进行中 {
  background: #fff3cd;
  color: #856404;
}

.status-badge.已完成 {
  background: #d4edda;
  color: #155724;
}

.toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  padding: 14px 24px;
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

.toast.success { background: var(--apple-green); color: white; }
.toast.error { background: var(--apple-red); color: white; }

.toast button {
  background: none;
  color: inherit;
  font-size: 18px;
  padding: 0;
  opacity: 0.7;
  border: none;
  cursor: pointer;
}

@keyframes slideUp {
  from { opacity: 0; transform: translateX(-50%) translateY(20px); }
  to { opacity: 1; transform: translateX(-50%) translateY(0); }
}

@media (max-width: 768px) {
  .form-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
