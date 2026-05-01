import axios from 'axios'

const api = axios.create({
  baseURL: import.meta.env.DEV ? '/api' : 'http://localhost:3000/api',
  timeout: 15000,
})

api.interceptors.response.use(
  (response) => response,
  (error) => {
    const message = error.response?.data?.error || error.message || '请求失败'
    console.error('API Error:', message)
    // 可以通过自定义事件通知 UI 层
    window.dispatchEvent(new CustomEvent('api-error', { detail: message }))
    return Promise.reject({ message, ...error.response?.data })
  }
)

export const pipesAPI = {
  list: (params = {}) => api.get('/pipes', { params }),
  get: (id) => api.get(`/pipes/${id}`),
  create: (data) => api.post('/pipes', data),
  update: (id, data) => api.put(`/pipes/${id}`, data),
  delete: (id) => api.delete(`/pipes/${id}`),
  batchDelete: (data) => api.post('/pipes/batch-delete', data),
  batchExport: (params = {}) => api.get('/pipes/batch-export', { params, responseType: 'blob' }),
  entry: (data) => api.post('/pipes/entry', data),
  exit: (data) => api.post('/pipes/exit', data),
}

export const recordsAPI = {
  list: (params = {}) => api.get('/records', { params }),
  create: (data) => api.post('/records', data),
}

export const productionAPI = {
  list: (params = {}) => api.get('/productions', { params }),
  create: (data) => api.post('/productions', data),
}

export const statsAPI = {
  overview: () => api.get('/statistics'),
  byMaterial: () => api.get('/material-stats'),
  lowStock: (threshold = 10) => api.get('/low-stock', { params: { threshold } }),
  trends: () => api.get('/trends'),
}

export const dictsAPI = {
  all: () => api.get('/dicts'),
}

export const logsAPI = {
  list: (limit = 50) => api.get('/logs', { params: { limit } }),
}

export const exportAPI = {
  inventory: () => api.get('/export/inventory', { responseType: 'blob' }),
  inventoryExcel: () => api.get('/export/inventory/excel', { responseType: 'blob' }),
  records: (params = {}) => api.get('/export/records', { params, responseType: 'blob' }),
  recordsExcel: (params = {}) => api.get('/export/records/excel', { params, responseType: 'blob' }),
}

export const importAPI = {
  csv: (data) => api.post('/import/csv', data),
  excel: (data) => api.post('/import/excel', data),
}

export default api
