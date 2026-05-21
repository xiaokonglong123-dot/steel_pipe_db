import { useMutation } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { authApi } from '../api/authApi';
import { useAuthStore } from '@/stores/authStore';

export function useLogin() {
  const setAuth = useAuthStore((s) => s.setAuth);
  const navigate = useNavigate();

  return useMutation({
    mutationFn: authApi.login,
    onSuccess: (data) => {
      setAuth(data.token, data.user);
      navigate('/');
    },
  });
}

export function useLogout() {
  const logout = useAuthStore((s) => s.logout);
  const navigate = useNavigate();

  return () => {
    logout();
    navigate('/login');
  };
}
