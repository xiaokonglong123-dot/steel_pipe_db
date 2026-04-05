<template>
  <div class="page">
    <div class="card">
      <form @submit.prevent="submitProduction" class="form">
        <h2 class="form-title">生产投料</h2>
        <div class="form-grid">
          <div class="form-group">
            <label>炉号 <span class="required">*</span></label>
            <input v-model="form.furnace_number" placeholder="请输入炉号" required />
          </div>
          <div class="form-group">
            <label>热处理批号</label>
            <input v-model="form.heat_treatment_batch" placeholder="热处理批号" />
          </div>
          <div class="form-group">
            <label>原料批号</label>
            <input v-model="form.material_batch" placeholder="原料批号" />
          </div>
          <div class="form-group">
            <label>投产支数 <span class="required">*</span></label>
            <input v-model.number="form.production_count" type="number" min="1" placeholder="投产支数" required />
          </div>
          <div class="form-group">
            <label>取样</label>
            <input v-model="form.sample" placeholder="取样信息" />
          </div>
          <div class="form-group">
            <label>供应商</label>
            <input v-model="form.supplier" placeholder="供应商" />
          </div>
          <div class="form-group">
            <label>操作员 <span class="required">*</span></label>
            <input v-model="form.operator" placeholder="操作员" required />
          </div>
        </div>
        <div class="form-group full">
          <label>备注</label>
          <textarea v-model="form.remarks" rows="3" placeholder="备注信息"></textarea>
        </div>
        <div class="form-actions">
          <button type="submit" class="btn-primary" :disabled="loading">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="19" x2="12" y2="5"/><polyline points="5,12 12,5 19,12"/>
            </svg>
            {{ loading ? '提交中...' : '确认投料' }}
          </button>
        </div>
      </form>
    </div>

    <div v-if="message" :class="['toast', messageType]">
      <svg v-if="messageType === 'success'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22,4 12,14.01 9,11.01"/>
      </svg>
      {{ message }}
      <button @click="message = ''">×</button>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive } from 'vue'
import { productionAPI } from '../api'

const form = reactive({
  furnace_number: '',
  heat_treatment_batch: '',
  material_batch: '',
  production_count: '',
  sample: '',
  supplier: '',
  operator: '',
  remarks: '',
})

const loading = ref(false)
const message = ref('')
const messageType = ref('')

async function submitProduction() {
  loading.value = true
  try {
    await productionAPI.create({
      furnace_number: form.furnace_number,
      heat_treatment_batch: form.heat_treatment_batch || null,
      material_batch: form.material_batch || null,
      production_count: form.production_count,
      sample: form.sample || null,
      supplier: form.supplier || null,
      operator: form.operator,
      remarks: form.remarks || null,
    })
    message.value = '投料成功！'
    messageType.value = 'success'
    Object.keys(form).forEach(k => form[k] = '')
  } catch (e) {
    message.value = e.message || '投料失败'
    messageType.value = 'error'
  } finally {
    loading.value = false
  }
}
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
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

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