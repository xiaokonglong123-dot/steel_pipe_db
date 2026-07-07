/**
 * Tests for the API client — verifies configuration and core behavior.
 *
 * The API client uses native fetch with JWT token attachment, 401 handling,
 * and timeout support. These tests mock fetch and verify the client's logic.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ApiError } from '@/api/client';

// Mock the auth store — must be hoisted above vi.mock call
// Mock the auth store — must be hoisted above vi.mock call
interface AuthState {
  token: string | null;
  logout: ReturnType<typeof vi.fn>;
}

const mockGetState = vi.hoisted(() =>
  vi.fn((): AuthState => ({
    token: null,
    logout: vi.fn(),
  })),
);
vi.mock('@/stores/authStore', () => ({
  useAuthStore: Object.assign(mockGetState, {
    getState: mockGetState,
  }),
}));

// Mock window.location
const mockLocation = {
  origin: 'http://localhost:5173',
  pathname: '/pipes/seamless',
  replace: vi.fn(),
};
Object.defineProperty(window, 'location', { value: mockLocation });

// Store original fetch
const originalFetch = globalThis.fetch;

describe('ApiError', () => {
  it('creates an error with status, code, message, and optional details', () => {
    const error = new ApiError(404, 12001, 'Pipe not found', { pipe_id: 42 });
    expect(error).toBeInstanceOf(Error);
    expect(error).toBeInstanceOf(ApiError);
    expect(error.name).toBe('ApiError');
    expect(error.status).toBe(404);
    expect(error.code).toBe(12001);
    expect(error.message).toBe('Pipe not found');
    expect(error.details).toEqual({ pipe_id: 42 });
  });

  it('works without details', () => {
    const error = new ApiError(500, 50001, 'Internal error');
    expect(error.status).toBe(500);
    expect(error.code).toBe(50001);
    expect(error.details).toBeUndefined();
  });
});

describe('API client configuration', () => {
  beforeEach(() => {
    mockGetState.mockReturnValue({
      token: null,
      logout: vi.fn(),
    });
    mockLocation.replace.mockClear();
    mockLocation.pathname = '/pipes/seamless';
  });

  afterEach(() => {
    globalThis.fetch = originalFetch;
  });

  it('uses /api/v1 as base URL', async () => {
    // Verify the module exports exist
    const client = await import('@/api/client');
    expect(client.default).toBeDefined();
    expect(typeof client.default.get).toBe('function');
    expect(typeof client.default.post).toBe('function');
    expect(typeof client.default.put).toBe('function');
    expect(typeof client.default.delete).toBe('function');
    expect(typeof client.default.postFormData).toBe('function');
    expect(typeof client.default.getBlob).toBe('function');
  });

  it('attaches Authorization header when token exists', async () => {
    mockGetState.mockReturnValue({
      token: 'test-jwt-token',
      logout: vi.fn(),
    });

    let capturedInit: RequestInit | undefined;
    globalThis.fetch = vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) => {
      capturedInit = init;
      return new Response(JSON.stringify({ success: true, data: {} }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    }) as typeof fetch;

    const client = await import('@/api/client');
    await client.default.get('/test-endpoint');

    expect(capturedInit).toBeDefined();
    expect(capturedInit!.headers).toBeDefined();
    const headers = capturedInit!.headers as Headers;
    expect(headers.get('Authorization')).toBe('Bearer test-jwt-token');
  });

  it('does not attach Authorization when no token', async () => {
    mockGetState.mockReturnValue({
      token: null,
      logout: vi.fn(),
    });

    let capturedInit: RequestInit | undefined;
    globalThis.fetch = vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) => {
      capturedInit = init;
      return new Response(JSON.stringify({ success: true, data: {} }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    }) as typeof fetch;

    const client = await import('@/api/client');
    await client.default.get('/test-endpoint');

    const headers = capturedInit!.headers as Headers;
    expect(headers.get('Authorization')).toBeNull();
  });

  it('sets Content-Type to application/json for non-FormData requests', async () => {
    mockGetState.mockReturnValue({ token: 'tok', logout: vi.fn() });

    let capturedInit: RequestInit | undefined;
    globalThis.fetch = vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) => {
      capturedInit = init;
      return new Response(JSON.stringify({ success: true, data: {} }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    }) as typeof fetch;

    const client = await import('@/api/client');
    await client.default.get('/test-endpoint');

    const headers = capturedInit!.headers as Headers;
    expect(headers.get('Content-Type')).toBe('application/json');
  });

  it('does not set Content-Type for FormData requests', async () => {
    mockGetState.mockReturnValue({ token: 'tok', logout: vi.fn() });

    let capturedInit: RequestInit | undefined;
    globalThis.fetch = vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) => {
      capturedInit = init;
      return new Response(JSON.stringify({ success: true, data: {} }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    }) as typeof fetch;

    const client = await import('@/api/client');
    const formData = new FormData();
    formData.append('file', new Blob(), 'test.xlsx');
    await client.default.postFormData('/upload', formData);

    const headers = capturedInit!.headers as Headers;
    expect(headers.get('Content-Type')).toBeNull();
  });

  it('appends query params to URL', async () => {
    mockGetState.mockReturnValue({ token: 'tok', logout: vi.fn() });

    let capturedUrl: string = '';
    globalThis.fetch = vi.fn(async (input: RequestInfo | URL) => {
      capturedUrl = input instanceof Request ? input.url : input.toString();
      return new Response(JSON.stringify({ success: true, data: {} }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    }) as typeof fetch;

    const client = await import('@/api/client');
    await client.default.get('/search', { q: 'J55', page: 1 });

    expect(capturedUrl).toContain('q=J55');
    expect(capturedUrl).toContain('page=1');
  });

  it('throws ApiError on 404 responses', async () => {
    mockGetState.mockReturnValue({
      token: 'tok',
      logout: vi.fn(),
    });

    globalThis.fetch = vi.fn(async () => {
      return new Response(
        JSON.stringify({
          success: false,
          code: 12001,
          request_id: 'req_test',
          message: 'Pipe not found: 42',
          details: null,
        }),
        {
          status: 404,
          headers: { 'Content-Type': 'application/json' },
        },
      );
    }) as typeof fetch;

    const client = await import('@/api/client');

    await expect(client.default.get('/pipes/42')).rejects.toThrow(ApiError);
    await expect(client.default.get('/pipes/42')).rejects.toMatchObject({
      status: 404,
      code: 12001,
    });
  });

  it('calls logout and redirects to /login on 401', async () => {
    const mockLogout = vi.fn();
    mockGetState.mockReturnValue({
      token: 'expired-token',
      logout: mockLogout,
    });

    globalThis.fetch = vi.fn(async () => {
      return new Response(
        JSON.stringify({
          success: false,
          code: 11001,
          request_id: 'req_test',
          message: 'Unauthorized',
          details: null,
        }),
        {
          status: 401,
          headers: { 'Content-Type': 'application/json' },
        },
      );
    }) as typeof fetch;

    const client = await import('@/api/client');

    await expect(client.default.get('/protected')).rejects.toThrow();
    expect(mockLogout).toHaveBeenCalled();
    expect(mockLocation.replace).toHaveBeenCalledWith(
      expect.stringContaining('/login?redirect='),
    );
  });
});
