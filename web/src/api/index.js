import axios from 'axios'

const api = axios.create({
  baseURL: import.meta.env.DEV ? '/api' : 'http://localhost:3000/api',
  timeout: 10000,
})

api.interceptors.response.use(
  (response) => response,
  (error) => {
    console.error('API Error:', error.response?.data || error.message)
    return Promise.reject(error)
  }
)

export const pipesAPI = {
  list: (params = {}) => api.get('/pipes', { params }),
  get: (id) => api.get(`/pipes/${id}`),
  create: (data) => api.post('/pipes', data),
  update: (id, data) => api.put(`/pipes/${id}`, data),
  delete: (id) => api.delete(`/pipes/${id}`),
  entry: (data) => api.post('/pipes/entry', data),
  exit: (data) => api.post('/pipes/exit', data),
}

export const recordsAPI = {
  list: (params = {}) => api.get('/records', { params }),
  create: (data) => api.post('/records', data),
}

export const statsAPI = {
  overview: () => api.get('/statistics'),
  byMaterial: () => api.get('/material-stats'),
  lowStock: (threshold = 10) => api.get('/low-stock', { params: { threshold } }),
}

export const logsAPI = {
  list: (limit = 50) => api.get('/logs', { params: { limit } }),
  undo: (id) => api.delete(`/undo/${id}`),
}

export const exportAPI = {
  inventory: () => api.get('/export/inventory', { responseType: 'blob' }),
  records: (params = {}) => api.get('/export/records', { params, responseType: 'blob' }),
}

export const importAPI = {
  csv: (data) => api.post('/import/csv', data),
}

export default api
