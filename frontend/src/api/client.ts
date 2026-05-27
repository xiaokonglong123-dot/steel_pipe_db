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
  const token = useAuthStore.getState().token;
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      useAuthStore.getState().logout();
      // Preserve current path so user can be redirected back after re-login
      const currentPath = window.location.pathname;
      const loginPath = currentPath && currentPath !== '/login'
        ? `/login?redirect=${encodeURIComponent(currentPath)}`
        : '/login';
      // Use replace to avoid back-button loop; only redirect if not already on login page
      if (window.location.pathname !== '/login') {
        window.location.replace(loginPath);
      }
    }
    return Promise.reject(error);
  },
);

export default apiClient;