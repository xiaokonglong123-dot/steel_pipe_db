<template>
  <div class="page">
    <div class="card">
      <form @submit.prevent="submitEntry" class="form">
        <h2 class="form-title">钢管入库</h2>
        <div class="form-grid">
          <div class="form-group">
            <label>钢管编号 <span class="required">*</span></label>
            <input v-model="form.pipe_id" placeholder="请输入钢管编号" required />
          </div>
          <div class="form-group">
            <label>材质 <span class="required">*</span></label>
            <select v-model="form.material" required>
              <option value="">选择材质</option>
              <option v-for="m in materials" :key="m" :value="m">{{ m }}</option>
            </select>
          </div>
          <div class="form-group">
            <label>直径 (mm) <span class="required">*</span></label>
            <input v-model.number="form.diameter" type="number" step="0.01" min="0" placeholder="直径" required />
          </div>
          <div class="form-group">
            <label>壁厚 (mm) <span class="required">*</span></label>
            <input v-model.number="form.thickness" type="number" step="0.01" min="0" placeholder="壁厚" required />
          </div>
          <div class="form-group">
            <label>长度 (m) <span class="required">*</span></label>
            <input v-model.number="form.length" type="number" step="0.01" min="0" placeholder="长度" required />
          </div>
          <div class="form-group">
            <label>数量 <span class="required">*</span></label>
            <input v-model.number="form.quantity" type="number" min="1" placeholder="数量" required />
          </div>
          <div class="form-group">
            <label>炉号</label>
            <input v-model="form.furnace_number" placeholder="炉号" />
          </div>
          <div class="form-group">
            <label>热处理批号</label>
            <input v-model="form.heat_treatment_batch" placeholder="热处理批号" />
          </div>
          <div class="form-group">
            <label>取样号</label>
            <input v-model="form.sample_number" placeholder="取样号" />
          </div>
          <div class="form-group">
            <label>投产支数</label>
            <input v-model.number="form.production_count" type="number" min="0" placeholder="投产支数" />
          </div>
          <div class="form-group">
            <label>原料架</label>
            <input v-model="form.material_rack" placeholder="原料架" />
          </div>
          <div class="form-group">
            <label>存放位置</label>
            <input v-model="form.location" placeholder="存放位置" />
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
            {{ loading ? '提交中...' : '确认入库' }}
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
import { pipesAPI } from '../api'

const materials = ['碳钢', '不锈钢', '合金钢', '无缝钢管', '焊接钢管']

const form = reactive({
  pipe_id: '', diameter: '', thickness: '', length: '',
  material: '', quantity: '', location: '', supplier: '',
  operator: '', remarks: '',
  furnace_number: '', heat_treatment_batch: '',
  sample_number: '', production_count: '', material_rack: '',
})

const loading = ref(false)
const message = ref('')
const messageType = ref('')

async function submitEntry() {
  loading.value = true
  try {
    await pipesAPI.entry({
      pipe: {
        pipe_id: form.pipe_id,
        diameter: form.diameter,
        thickness: form.thickness,
        length: form.length,
        material: form.material,
        quantity: form.quantity,
        location: form.location || null,
        supplier: form.supplier || null,
        entry_date: '', last_update: null, status: '在库',
        furnace_number: form.furnace_number || null,
        heat_treatment_batch: form.heat_treatment_batch || null,
        sample_number: form.sample_number || null,
        production_count: form.production_count || null,
        material_rack: form.material_rack || null,
        remarks: form.remarks || null,
      },
      operator: form.operator,
      remarks: form.remarks || null,
    })
    message.value = '入库操作成功！'
    messageType.value = 'success'
    Object.keys(form).forEach(k => form[k] = '')
  } catch (e) {
    message.value = e.message || '入库失败'
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
