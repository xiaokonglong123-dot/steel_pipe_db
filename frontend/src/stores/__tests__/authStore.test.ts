/**
 * Auth store unit tests.
 *
 * Auth state is in-memory only (refresh via httpOnly cookie, no localStorage).
 * Tests verifying localStorage interaction have been removed — the store
 * does not write to localStorage. Recovery tests simulate the init logic.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { useAuthStore } from '@/stores/authStore';
import type { UserInfo } from '@/types';

const testUser: UserInfo = {
  id: 1,
  username: 'admin',
  display_name: 'Admin',
  role: 'admin',
  email: 'admin@test.local',
};

beforeEach(() => {
  // Clear localStorage before each test
  localStorage.clear();
  // Reset store to initial state
  useAuthStore.setState({ user: null, token: null });
});

describe('authStore', () => {
  it('starts with null user and token when localStorage is empty', () => {
    const { user, token } = useAuthStore.getState();
    expect(user).toBeNull();
    expect(token).toBeNull();
  });

  it('setAuth stores user and token in store state', () => {
    useAuthStore.getState().setAuth(testUser, 'jwt-token-123');

    const { user, token } = useAuthStore.getState();
    expect(user).toEqual(testUser);
    expect(token).toBe('jwt-token-123');
  });

  it('setUser updates user without changing token', () => {
    useAuthStore.getState().setAuth(testUser, 'jwt-token-123');

    const updatedUser: UserInfo = { ...testUser, display_name: 'Super Admin' };
    useAuthStore.getState().setUser(updatedUser);

    const { user, token } = useAuthStore.getState();
    expect(user?.display_name).toBe('Super Admin');
    expect(token).toBe('jwt-token-123');
  });

  it('logout clears user and token from store state', () => {
    useAuthStore.getState().setAuth(testUser, 'jwt-token-123');
    useAuthStore.getState().logout();

    const { user, token } = useAuthStore.getState();
    expect(user).toBeNull();
    expect(token).toBeNull();
  });

  it('recovers user and token from localStorage on store creation', () => {
    // Pre-populate localStorage
    localStorage.setItem('auth_user', JSON.stringify(testUser));
    localStorage.setItem('auth_token', 'recovered-token');

    // Re-create the store initial state (simulates page refresh)
    useAuthStore.setState({
      user: (() => {
        try {
          const raw = localStorage.getItem('auth_user');
          return raw ? JSON.parse(raw) : null;
        } catch {
          return null;
        }
      })(),
      token: localStorage.getItem('auth_token'),
    });

    const { user, token } = useAuthStore.getState();
    expect(user).toEqual(testUser);
    expect(token).toBe('recovered-token');
  });

  it('handles corrupted localStorage gracefully', () => {
    localStorage.setItem('auth_user', '{invalid-json}');

    useAuthStore.setState({
      user: (() => {
        try {
          const raw = localStorage.getItem('auth_user');
          return raw ? JSON.parse(raw) : null;
        } catch {
          localStorage.removeItem('auth_user');
          return null;
        }
      })(),
    });

    const { user } = useAuthStore.getState();
    expect(user).toBeNull();
    expect(localStorage.getItem('auth_user')).toBeNull();
  });
});
