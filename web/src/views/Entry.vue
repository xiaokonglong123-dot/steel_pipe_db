<template>
  <div class="page">
    <h1 class="page-title">钢管入库</h1>

    <div class="card">
      <form @submit.prevent="submitEntry" class="form">
        <div class="form-grid">
          <div class="form-group">
            <label>钢管编号</label>
            <input v-model="form.pipe_id" placeholder="请输入钢管编号" required />
          </div>
          <div class="form-group">
            <label>材质</label>
            <select v-model="form.material" required>
              <option value="">选择材质</option>
              <option v-for="m in materials" :key="m" :value="m">{{ m }}</option>
            </select>
          </div>
          <div class="form-group">
            <label>直径 (mm)</label>
            <input v-model.number="form.diameter" type="number" step="0.01" min="0" placeholder="直径" required />
          </div>
          <div class="form-group">
            <label>壁厚 (mm)</label>
            <input v-model.number="form.thickness" type="number" step="0.01" min="0" placeholder="壁厚" required />
          </div>
          <div class="form-group">
            <label>长度 (m)</label>
            <input v-model.number="form.length" type="number" step="0.01" min="0" placeholder="长度" required />
          </div>
          <div class="form-group">
            <label>数量</label>
            <input v-model.number="form.quantity" type="number" min="1" placeholder="数量" required />
          </div>
          <div class="form-group">
            <label>存放位置</label>
            <input v-model="form.location" placeholder="存放位置（可选）" />
          </div>
          <div class="form-group">
            <label>供应商</label>
            <input v-model="form.supplier" placeholder="供应商（可选）" />
          </div>
          <div class="form-group">
            <label>操作员</label>
            <input v-model="form.operator" placeholder="操作员" required />
          </div>
        </div>
        <div class="form-group full">
          <label>备注</label>
          <textarea v-model="form.remarks" rows="2" placeholder="备注（可选）"></textarea>
        </div>
        <div class="form-actions">
          <button type="submit" class="btn-primary" :disabled="loading">
            {{ loading ? '提交中...' : '确认入库' }}
          </button>
        </div>
      </form>
    </div>

    <div v-if="message" :class="['toast', messageType]">
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
      },
      operator: form.operator,
      remarks: form.remarks || null,
    })
    message.value = '入库操作成功！'
    messageType.value = 'success'
    Object.keys(form).forEach(k => form[k] = '')
  } catch (e) {
    message.value = e.response?.data?.error || '入库失败'
    messageType.value = 'error'
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.page-title {
  font-size: 34px;
  font-weight: 700;
  letter-spacing: -0.02em;
  margin-bottom: 28px;
}

.card {
  background: var(--apple-card);
  border-radius: var(--apple-radius);
  padding: 32px;
  box-shadow: var(--apple-shadow);
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

label {
  font-size: 13px;
  font-weight: 500;
  color: var(--apple-text-secondary);
}

input, select, textarea {
  padding: 10px 14px;
  border: 1px solid var(--apple-border);
  border-radius: var(--apple-radius-sm);
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
  margin-top: 24px;
}

.btn-primary {
  background: var(--apple-blue);
  color: white;
  padding: 12px 32px;
  border-radius: 980px;
  font-size: 15px;
  font-weight: 500;
  transition: var(--apple-transition);
}

.btn-primary:hover { background: var(--apple-blue-hover); }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

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

.toast.success { background: #1d1d1f; color: white; }
.toast.error { background: var(--apple-red); color: white; }

.toast button {
  background: none;
  color: inherit;
  font-size: 18px;
  padding: 0;
  opacity: 0.7;
}

@keyframes slideUp {
  from { opacity: 0; transform: translateX(-50%) translateY(20px); }
  to { opacity: 1; transform: translateX(-50%) translateY(0); }
}
</style>
