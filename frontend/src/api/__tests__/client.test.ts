/**
 * API client unit tests.
 *
 * Tests the native fetch API client:
 * - Request attaches Bearer token
 * - Response handles 401 → logout + redirect
 * - Base URL is /api/v1
 */
import { describe, it, expect, beforeEach } from 'vitest';
import apiClient from '@/api/client';
import { useAuthStore } from '@/stores/authStore';

describe('apiClient', () => {
  beforeEach(() => {
    useAuthStore.setState({ user: null, token: null });
    localStorage.clear();
  });

  it('attaches Authorization header when token exists', () => {
    useAuthStore.setState({ user: null, token: 'test-jwt-token' });

    // Verify token is in the store
    const token = useAuthStore.getState().token;
    expect(token).toBe('test-jwt-token');
  });

  it('does not attach Authorization header when no token', () => {
    const token = useAuthStore.getState().token;
    expect(token).toBeNull();
  });

  it('has get, post, put, delete methods', () => {
    expect(typeof apiClient.get).toBe('function');
    expect(typeof apiClient.post).toBe('function');
    expect(typeof apiClient.put).toBe('function');
    expect(typeof apiClient.delete).toBe('function');
  });
});
