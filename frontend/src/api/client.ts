/**
 * Native fetch API client — replaces Axios
 *
 * Features:
 * - Automatic JWT token attachment
 * - 401 handling with redirect to login
 * - Type-safe request/response
 * - Support for JSON and FormData bodies
 * - Support for JSON and Blob responses
 */
import { useAuthStore } from '@/stores/authStore';

const BASE_URL = '/api/v1';
const TIMEOUT_MS = 30_000;

/** Custom error class for API errors */
export class ApiError extends Error {
  constructor(
    public status: number,
    public code: number,
    message: string,
    public details?: unknown,
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

/** Build headers with optional auth token */
function buildHeaders(token?: string | null, isFormData?: boolean): Headers {
  const headers = new Headers();
  if (!isFormData) {
    headers.set('Content-Type', 'application/json');
  }
  if (token) {
    headers.set('Authorization', `Bearer ${token}`);
  }
  return headers;
}

/** Handle 401 responses — clear auth and redirect */
function handle401(): never {
  const { logout } = useAuthStore.getState();
  logout();
  const currentPath = window.location.pathname;
  const loginPath =
    currentPath && currentPath !== '/login'
      ? `/login?redirect=${encodeURIComponent(currentPath)}`
      : '/login';
  if (window.location.pathname !== '/login') {
    window.location.replace(loginPath);
  }
  throw new ApiError(401, 11001, 'Unauthorized');
}

let refreshPromise: Promise<string | null> | null = null;

/**
 * Try to refresh the access token via the httpOnly refresh cookie.
 * Concurrent 401s share a single refresh call (single-flight).
 * Returns the new token, or null when the refresh fails.
 */
function refreshAccessToken(): Promise<string | null> {
  if (!refreshPromise) {
    refreshPromise = (async () => {
      try {
        const res = await fetch(`${BASE_URL}/auth/refresh`, {
          method: 'POST',
          credentials: 'include',
          headers: { 'Content-Type': 'application/json' },
        });
        if (!res.ok) return null;
        const body = await res.json();
        const token: unknown = body?.data?.token;
        return typeof token === 'string' && token.length > 0 ? token : null;
      } catch {
        return null;
      } finally {
        refreshPromise = null;
      }
    })();
  }
  return refreshPromise;
}

interface RequestOptions {
  body?: unknown;
  params?: object;
  signal?: AbortSignal;
  responseType?: 'json' | 'blob';
}

/** Core fetch wrapper with timeout and error handling */
async function request<T>(
  method: string,
  path: string,
  options?: RequestOptions,
): Promise<T> {
  const { token } = useAuthStore.getState();

  // Build URL with query params
  const url = new URL(`${BASE_URL}${path}`, window.location.origin);
  if (options?.params) {
    Object.entries(options.params).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        url.searchParams.set(key, String(value));
      }
    });
  }

  // Create abort controller for timeout
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), TIMEOUT_MS);

  // Combine external signal with timeout
  const signal = options?.signal
    ? AbortSignal.any([controller.signal, options.signal])
    : controller.signal;

  // Check if body is FormData
  const isFormData = options?.body instanceof FormData;

  const perform = (authToken: string | null) =>
    fetch(url.toString(), {
      method,
      headers: buildHeaders(authToken, isFormData),
      body: options?.body
        ? isFormData
          ? (options.body as FormData)
          : JSON.stringify(options.body)
        : undefined,
      signal,
      credentials: 'include',
    });

  try {
    let response = await perform(token);

    // On 401, try a single-flight token refresh and replay the request once.
    const isAuthEndpoint = path === '/auth/refresh' || path === '/auth/login';
    if (response.status === 401 && !isAuthEndpoint) {
      const newToken = await refreshAccessToken();
      if (newToken) {
        useAuthStore.getState().setToken(newToken);
        response = await perform(newToken);
      }
    }

    clearTimeout(timeoutId);

    // Handle 401 (refresh failed or auth endpoint itself returned 401)
    if (response.status === 401) {
      handle401();
    }

    // Handle blob responses
    if (options?.responseType === 'blob') {
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new ApiError(
          response.status,
          errorData.code || response.status,
          errorData.message || `Request failed: ${response.statusText}`,
          errorData.details,
        );
      }
      return response.blob() as Promise<T>;
    }

    // Parse JSON response
    const data = await response.json();

    // Handle errors
    if (!response.ok) {
      throw new ApiError(
        response.status,
        data.code || response.status,
        data.message || `Request failed: ${response.statusText}`,
        data.details,
      );
    }

    return data as T;
  } catch (error) {
    clearTimeout(timeoutId);

    if (error instanceof ApiError) {
      throw error;
    }

    // Handle abort
    if (error instanceof DOMException && error.name === 'AbortError') {
      throw new ApiError(408, 0, 'Request timeout');
    }

    // Network errors
    throw new ApiError(0, 0, `Network error: ${(error as Error).message}`);
  }
}

/** HTTP client methods */
export const apiClient = {
  get: <T>(
    path: string,
    params?: object,
    signal?: AbortSignal,
  ) => request<T>('GET', path, { params, signal }),

  post: <T>(
    path: string,
    body?: unknown,
    signal?: AbortSignal,
  ) => request<T>('POST', path, { body, signal }),

  put: <T>(
    path: string,
    body?: unknown,
    signal?: AbortSignal,
  ) => request<T>('PUT', path, { body, signal }),

  delete: <T>(path: string, signal?: AbortSignal) =>
    request<T>('DELETE', path, { signal }),

  /** POST with FormData body (for file uploads) */
  postFormData: <T>(
    path: string,
    formData: FormData,
    signal?: AbortSignal,
  ) => request<T>('POST', path, { body: formData, signal }),

  /** GET with blob response (for file downloads) */
  getBlob: (
    path: string,
    params?: object,
    signal?: AbortSignal,
  ) => request<Blob>('GET', path, { params, signal, responseType: 'blob' }),
};

export default apiClient;
