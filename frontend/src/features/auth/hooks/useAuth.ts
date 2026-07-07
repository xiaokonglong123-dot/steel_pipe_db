import { useEffect } from 'react';
import { useMutation } from '@tanstack/react-query';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { authApi } from '../api/authApi';
import { useAuthStore } from '@/stores/authStore';

export function useLogin() {
  const setAuth = useAuthStore((s) => s.setAuth);
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  return useMutation({
    mutationFn: authApi.login,
    onSuccess: (data) => {
      setAuth(data.user, data.token);
      const redirect = searchParams.get('redirect') || '/';
      navigate(redirect);
    },
  });
}

export function useLogout() {
  const logout = useAuthStore((s) => s.logout);
  const navigate = useNavigate();

  return () => {
    authApi.logout().catch(() => {}).finally(() => {
      logout();
      navigate('/login');
    });
  };
}

export function useRestoreSession() {
  const setAuth = useAuthStore((s) => s.setAuth);
  const setRestoring = useAuthStore((s) => s.setRestoring);

  useEffect(() => {
    let cancelled = false;

    authApi.refresh()
      .then((data) => {
        if (!cancelled) {
          return authApi.getMe().then((user) => {
            if (!cancelled) {
              setAuth(user, data.token);
            }
          });
        }
      })
      .catch(() => {
        // Error is expected — token may be expired, user may not be logged in
      })
      .finally(() => {
        if (!cancelled) {
          setRestoring(false);
        }
      });

    return () => { cancelled = true; };
  }, [setAuth, setRestoring]);
}
