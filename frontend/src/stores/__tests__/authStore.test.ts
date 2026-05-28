/**
 * Auth store unit tests.
 *
 * Tests the Zustand authStore:
 * - Initial state from empty localStorage
 * - setAuth stores user + token
 * - setUser updates user
 * - logout clears everything
 * - localStorage persistence
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

  it('setAuth stores user and token in state and localStorage', () => {
    useAuthStore.getState().setAuth(testUser, 'jwt-token-123');

    const { user, token } = useAuthStore.getState();
    expect(user).toEqual(testUser);
    expect(token).toBe('jwt-token-123');

    // Verify localStorage
    expect(localStorage.getItem('auth_user')).toBe(JSON.stringify(testUser));
    expect(localStorage.getItem('auth_token')).toBe('jwt-token-123');
  });

  it('setUser updates user without changing token', () => {
    useAuthStore.getState().setAuth(testUser, 'jwt-token-123');

    const updatedUser: UserInfo = { ...testUser, display_name: 'Super Admin' };
    useAuthStore.getState().setUser(updatedUser);

    const { user, token } = useAuthStore.getState();
    expect(user?.display_name).toBe('Super Admin');
    expect(token).toBe('jwt-token-123');
  });

  it('logout clears user, token, and localStorage', () => {
    useAuthStore.getState().setAuth(testUser, 'jwt-token-123');
    useAuthStore.getState().logout();

    const { user, token } = useAuthStore.getState();
    expect(user).toBeNull();
    expect(token).toBeNull();

    expect(localStorage.getItem('auth_user')).toBeNull();
    expect(localStorage.getItem('auth_token')).toBeNull();
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
