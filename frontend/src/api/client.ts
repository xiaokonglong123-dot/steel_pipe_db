/**
 * Axios instance — global HTTP client
 *
 * baseURL points to /api/v1 (Vite dev proxy forwards to backend :3000),
 * 30s timeout, sends credentials automatically.
 *
 * Response interceptor: on 401, auto-cleans auth state and redirects to /login.
 */
import axios from 'axios';
import { useAuthStore } from '@/stores/authStore';

const apiClient = axios.create({
  baseURL: '/api/v1',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
  withCredentials: true,
});

apiClient.interceptors.request.use((config) => {
  return config;
});

apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      useAuthStore.getState().logout();
      window.location.href = '/login';
    }
    return Promise.reject(error);
  },
);

export default apiClient;